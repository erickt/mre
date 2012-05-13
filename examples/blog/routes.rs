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

impl render_200 for @response {
    fn render_200<T: to_mustache>(mu: mustache::context, path: str, data: T) {
        let data = alt check data.to_mustache() {
            mustache::map(m) { m }
        };

        let template = mu.render_file(path, data);
        self.http_200(str::bytes(template))
    }
}

fn routes(app: app::app) {
    // Show all the posts.
    app.get("^/$") { |_req, rep, _m|
        let posts = post::all(app.es);

        // This can be simplified after mozilla/rust/issues/2258 is fixed.
        rep.render_200(app.mu, "index", hash_from_strs([
            ("posts", posts.to_mustache())
        ]).to_mustache())
    }

    // Create a user.
    app.get("^/signup$") { |_req, rep, _m|
        let hash: hashmap<str, str> = str_hash();
        rep.render_200(app.mu, "signup", hash)
    }

    app.post("^/signup$") { |req, rep, _m|
        import user::user;

        let form = uri::decode_qs(req.body());
        alt form.find("username") {
          none { rep.http_400(str::bytes("missing username")) }
          some(username) {
            let username = username[0];

            alt form.find("password") {
              none { rep.http_400(str::bytes("missing password")) }
              some(password) {
                alt form.find("password_confirm") {
                  some(password_confirm) if password == password_confirm { 
                    let user = user::user(app.es, app.password_hasher, "blog",
                                          username, password[0]);

                    alt user.create() {
                      ok((id, _)) { rep.redirect("/") }
                      err(e) { rep.http_400(str::bytes(e)) }
                    }
                  }
                  _ {
                    rep.http_400(str::bytes("invalid password confirmation"))
                  }
                }
              }
            }
          }
        }
    }

    // Login a user.
    app.get("^/login$") { |_req, rep, _m|
        let hash: hashmap<str, str> = str_hash();
        rep.render_200(app.mu, "login", hash)
    }

    app.post("^/login$") { |req, rep, _m|
        import user::user;

        let form = uri::decode_qs(req.body());
        alt form.find("username") {
          none { rep.http_400(str::bytes("missing username")) }
          some(username) {
            let username = username[0];

            alt form.find("password") {
              none { rep.http_400(str::bytes("missing password")) }
              some(password) {
                let password = password[0];

                alt user::find(app.es, "blog", username) {
                  none { rep.http_400(str::bytes("user does not exist")) }
                  some(user) {
                    if app.password_hasher.verify(password, user.password()) {
                        rep.redirect("/")
                    } else {
                        rep.http_401()
                    }
                  }
                }
              }
            }
          }
        }
    }

    // Show all the users.
    app.get("^/users$") { |_req, rep, _m|
        let users = user::all(app.es, "blog");

        #error("%?", users);

        // This can be simplified after mozilla/rust/issues/2258 is fixed.
        rep.render_200(app.mu, "user_index", hash_from_strs([
            ("users", users.to_mustache())
        ]).to_mustache())
    }

    // Show a user.
    app.get("^/users/(?<id>[-_A-Za-z0-9]+)$") { |_req, rep, m|
        let id = m.named("id");

        alt user::find(app.es, "blog", id) {
          none { rep.http_404() }
          some(user) {
            rep.render_200(app.mu, "user_show", hash_from_strs([
                ("user", user.to_mustache())
            ]).to_mustache())
          }
        }
    }

    // Delete a user.
    app.post("^/users/(?<id>[-_A-Za-z0-9]+)/delete$") { |_req, rep, m|
        let id = m.named("id");

        alt user::find(app.es, "blog", id) {
          none { rep.http_404() }
          some(user) {
            user.delete();

            rep.redirect("/")
          }
        }
    }

    // Create a post.
    app.get("^/posts/new$") { |_req, rep, _m|
        rep.render_200(app.mu, "post_new", post::post(app.es, ""))
    }

    app.post("^/posts$") { |req, rep, _m|
        let form = uri::decode_qs(req.body());
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
          ok((id, _version)) { rep.redirect("/posts/" + id) }
          err(e) { rep.http_500(str::bytes(e)) }
        }
    }

    // Show a post.
    app.get("^/posts/(?<id>[-_A-Za-z0-9]+)$") { |_req, rep, m|
        let id = m.named("id");

        alt post::find(app.es, id) {
          none { rep.http_404() }
          some(post) {
            let comments = post.find_comments();

            rep.render_200(app.mu, "post_show", hash_from_strs([
                ("post_id", id.to_mustache()),
                ("post", post.to_mustache()),
                ("comments", comments.to_mustache())
            ]).to_mustache())
          }
        }
    }

    // Edit a post.
    app.get("^/posts/(?<id>[-_A-Za-z0-9]+)/edit$") { |_req, rep, m|
        let id = m.named("id");

        alt post::find(app.es, id) {
          none { rep.http_404() }
          some(post) {
            rep.render_200(app.mu, "post_edit", hash_from_strs([
                ("post_id", id.to_mustache()),
                ("post", post.to_mustache())
            ]).to_mustache())
          }
        }
    }

    app.post("^/posts/(?<id>[-_A-Za-z0-9]+)$") { |req, rep, m|
        let post_id = m.named("id");
        let form = uri::decode_qs(req.body());

        alt post::find(app.es, post_id) {
          none { rep.http_404() }
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
              ok(id) { rep.redirect("/posts/" + post_id) }
              err(e) { rep.http_500(str::bytes(e)) }
            }
          }
        }
    }

    // Delete a post.
    app.post("^/posts/(?<id>[-_A-Za-z0-9]+)/delete$") { |_req, rep, m|
        alt post::find(app.es, m.named("id")) {
          none { rep.http_404() }
          some(post) {
            post.delete();

            rep.redirect("/")
          }
        }
    }

    // Create a comment.
    app.post("^/posts/(?<id>[-_A-Za-z0-9]+)/comments$") { |req, rep, m|
        let id = m.named("id");
        let form = uri::decode_qs(req.body());

        alt post::find(app.es, id) {
          none { rep.http_404() }
          some(post) {
            let comment = comment::comment(app.es, id, "");

            alt form.find("body") {
              some(body) { comment.set_body(body[0]); }
              none {}
            }

            alt comment.save() {
              ok(_) { rep.redirect("/posts/" + id) }
              err(e){ rep.http_500(str::bytes(e)) }
            }
          }
        }
    }

    // Edit a comment.
    app.get("^/posts/(?<post_id>[-_A-Za-z0-9]+)/comments/(?<id>[-_A-Za-z0-9]+)/edit$") { |_req, rep, m|
        let post_id = m.named("post_id");
        let comment_id = m.named("id");

        alt comment::find(app.es, post_id, comment_id) {
          none { rep.http_404() }
          some(comment) {
            rep.render_200(app.mu, "comment_edit", hash_from_strs([
                ("post_id", post_id.to_mustache()),
                ("comment", comment.to_mustache())
            ]).to_mustache())
          }
        }
    }

    app.post("^/posts/(?<post_id>[-_A-Za-z0-9]+)/comments/(?<id>[-_A-Za-z0-9]+)$") { |req, rep, m|
        let post_id = m.named("post_id");
        let comment_id = m.named("id");
        let form = uri::decode_qs(req.body());

        alt comment::find(app.es, post_id, comment_id) {
          none { rep.http_404() }
          some(comment) {
            alt form.find("body") {
              some(body) { comment.set_body(body[0]); }
              none {}
            }

            alt comment.save() {
              ok(id) { rep.redirect("/posts/" + post_id) }
              err(e) { rep.http_500(str::bytes(e)) }
            }
          }
        }
    }

    // Delete a comment.
    app.post("^/posts/(?<post_id>[-_A-Za-z0-9]+)/comments/(?<id>[-_A-Za-z0-9]+)/delete$") { |_req, rep, m|
        let post_id = m.named("post_id");
        let comment_id = m.named("id");

        alt comment::find(app.es, post_id, comment_id) {
          none { rep.http_404() }
          some(comment) {
            comment.delete();

            rep.redirect("/posts/" + post_id)
          }
        }
    }
}
