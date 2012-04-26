import mre::response::{
    response,
    http_200,
    http_400,
    http_401,
    http_404,
    http_500,
    redirect
};
import mustache::context;

// FIXME: move after https://github.com/mozilla/rust/issues/2242 is fixed.
impl of to_mustache for user::user {
    fn to_mustache() -> mustache::data {
        import user::user;

        hash_from_strs([
            ("user_id", self.user_id())
        ]).to_mustache()
    }
}

impl of to_mustache for post::post {
    fn to_mustache() -> mustache::data {
        import post::post;

        hash_from_strs([
            ("post_id", self.post_id()),
            ("title", self.title()),
            ("body", self.body())
        ]).to_mustache()
    }
}

impl of to_mustache for comment::comment {
    fn to_mustache() -> mustache::data {
        import comment::comment;

        hash_from_strs([
            ("comment_id", self.comment_id()),
            ("body", self.body())
        ]).to_mustache()
    }
}

impl render_200 for app {
    fn render_200<T: to_mustache>(req: request, path: str, data: T) -> response {
        let data = alt check data.to_mustache() {
            mustache::map(m) { m }
        };

        let template = self.mu.render_file(path, data);
        http_200(req, str::bytes(template))
    }
}

fn routes(app: app::app) {
    // Show all the posts.
    app.get("^/$") { |req, _m|
        let posts = post::all(app.es);

        // This can be simplified after mozilla/rust/issues/2258 is fixed.
        app.render_200(req, "index", hash_from_strs([
            ("posts", posts.to_mustache())
        ]).to_mustache())
    }

    // Create a user.
    app.get("^/signup$") { |req, _m|
        let hash: hashmap<str, str> = str_hash();
        app.render_200(req, "signup", hash)
    }

    app.post("^/signup$") { |req, _m|
        import user::user;

        let form = uri::decode_qs(req.body);
        alt form.find("username") {
          none { http_400(req, str::bytes("missing username")) }
          some(username) {
            let username = username[0];

            alt form.find("password") {
              none { http_400(req, str::bytes("missing password")) }
              some(password) {
                alt form.find("password_confirm") {
                  some(password_confirm) if password == password_confirm { 
                    let user = user::user(app.es, app.password_hasher, "blog",
                                          username, password[0]);

                    alt user.create() {
                      ok((id, _)) { redirect(req, "/") }
                      err(e) { http_400(req, str::bytes(e)) }
                    }
                  }
                  _ {
                    http_400(req, str::bytes("invalid password confirmation"))
                  }
                }
              }
            }
          }
        }
    }

    // Login a user.
    app.get("^/login$") { |req, _m|
        let hash: hashmap<str, str> = str_hash();
        app.render_200(req, "login", hash)
    }

    app.post("^/login$") { |req, _m|
        import user::user;

        let form = uri::decode_qs(req.body);
        alt form.find("username") {
          none { http_400(req, str::bytes("missing username")) }
          some(username) {
            let username = username[0];

            alt form.find("password") {
              none { http_400(req, str::bytes("missing password")) }
              some(password) {
                let password = password[0];

                alt user::find(app.es, "blog", username) {
                  none { http_400(req, str::bytes("user does not exist")) }
                  some(user) {
                    if app.password_hasher.verify(password, user.password()) {
                        redirect(req, "/")
                    } else {
                        http_401(req)
                    }
                  }
                }
              }
            }
          }
        }
    }

    // Show all the users.
    app.get("^/users$") { |req, _m|
        let users = user::all(app.es, "blog");

        #error("%?", users);

        // This can be simplified after mozilla/rust/issues/2258 is fixed.
        app.render_200(req, "user_index", hash_from_strs([
            ("users", users.to_mustache())
        ]).to_mustache())
    }

    // Show a user.
    app.get("^/users/(?<id>[-_A-Za-z0-9]+)$") { |req, m|
        let id = m.named("id");

        alt user::find(app.es, "blog", id) {
          none { http_404(req) }
          some(user) {
            app.render_200(req, "user_show", hash_from_strs([
                ("user", user.to_mustache())
            ]).to_mustache())
          }
        }
    }

    // Delete a user.
    app.post("^/users/(?<id>[-_A-Za-z0-9]+)/delete$") { |req, m|
        let id = m.named("id");

        alt user::find(app.es, "blog", id) {
          none { http_404(req) }
          some(user) {
            user.delete();

            redirect(req, "/")
          }
        }
    }

    // Create a post.
    app.get("^/posts/new$") { |req, _m|
        app.render_200(req, "post_new", post::post(app.es, ""))
    }

    app.post("^/posts$") { |req, _m|
        let form = uri::decode_qs(req.body);
        let post = post::post(app.es, "");

        alt form.find("title") {
          some(title) { post.set_title(title[0]); }
          none {}
        }

        alt form.find("body") {
          some(body) { post.set_body(body[0]); }
          none {}
        }

        alt post.save() {
          ok((id, _version)) { redirect(req, "/posts/" + id) }
          err(e) { http_500(req, str::bytes(e)) }
        }
    }

    // Show a post.
    app.get("^/posts/(?<id>[-_A-Za-z0-9]+)$") { |req, m|
        let id = m.named("id");

        alt post::find(app.es, id) {
          none { http_404(req) }
          some(post) {
            let comments = post.find_comments();

            app.render_200(req, "post_show", hash_from_strs([
                ("post_id", id.to_mustache()),
                ("post", post.to_mustache()),
                ("comments", comments.to_mustache())
            ]).to_mustache())
          }
        }
    }

    // Edit a post.
    app.get("^/posts/(?<id>[-_A-Za-z0-9]+)/edit$") { |req, m|
        let id = m.named("id");

        alt post::find(app.es, id) {
          none { http_404(req) }
          some(post) {
            app.render_200(req, "post_edit", hash_from_strs([
                ("post_id", id.to_mustache()),
                ("post", post.to_mustache())
            ]).to_mustache())
          }
        }
    }

    app.post("^/posts/(?<id>[-_A-Za-z0-9]+)$") { |req, m|
        let post_id = m.named("id");
        let form = uri::decode_qs(req.body);

        alt post::find(app.es, post_id) {
          none { http_404(req) }
          some(post) {
            alt form.find("title") {
              some(title) { post.set_title(title[0]); }
              none {}
            }

            alt form.find("body") {
              some(body) { post.set_body(body[0]); }
              none {}
            }

            alt post.save() {
              ok(id) { redirect(req, "/posts/" + post_id) }
              err(e) { http_500(req, str::bytes(e)) }
            }
          }
        }
    }

    // Delete a post.
    app.post("^/posts/(?<id>[-_A-Za-z0-9]+)/delete$") { |req, m|
        alt post::find(app.es, m.named("id")) {
          none { http_404(req) }
          some(post) {
            post.delete();

            redirect(req, "/")
          }
        }
    }

    // Create a comment.
    app.post("^/posts/(?<id>[-_A-Za-z0-9]+)/comments$") { |req, m|
        let id = m.named("id");
        let form = uri::decode_qs(req.body);

        alt post::find(app.es, id) {
          none { http_404(req) }
          some(post) {
            let comment = comment::comment(app.es, id, "");

            alt form.find("body") {
              some(body) { comment.set_body(body[0]); }
              none {}
            }

            alt comment.save() {
              ok(_) { redirect(req, "/posts/" + id) }
              err(e){ http_500(req, str::bytes(e)) }
            }
          }
        }
    }

    // Edit a comment.
    app.get("^/posts/(?<post_id>[-_A-Za-z0-9]+)/comments/(?<id>[-_A-Za-z0-9]+)/edit$") { |req, m|
        let post_id = m.named("post_id");
        let comment_id = m.named("id");

        alt comment::find(app.es, post_id, comment_id) {
          none { http_404(req) }
          some(comment) {
            app.render_200(req, "comment_edit", hash_from_strs([
                ("post_id", post_id.to_mustache()),
                ("comment", comment.to_mustache())
            ]).to_mustache())
          }
        }
    }

    app.post("^/posts/(?<post_id>[-_A-Za-z0-9]+)/comments/(?<id>[-_A-Za-z0-9]+)$") { |req, m|
        let post_id = m.named("post_id");
        let comment_id = m.named("id");
        let form = uri::decode_qs(req.body);

        alt comment::find(app.es, post_id, comment_id) {
          none { http_404(req) }
          some(comment) {
            alt form.find("body") {
              some(body) { comment.set_body(body[0]); }
              none {}
            }

            alt comment.save() {
              ok(id) { redirect(req, "/posts/" + post_id) }
              err(e) { http_500(req, str::bytes(e)) }
            }
          }
        }
    }

    // Delete a comment.
    app.post("^/posts/(?<post_id>[-_A-Za-z0-9]+)/comments/(?<id>[-_A-Za-z0-9]+)/delete$") { |req, m|
        let post_id = m.named("post_id");
        let comment_id = m.named("id");

        alt comment::find(app.es, post_id, comment_id) {
          none { http_404(req) }
          some(comment) {
            comment.delete();

            redirect(req, "/posts/" + post_id)
          }
        }
    }
}
