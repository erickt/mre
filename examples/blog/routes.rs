import mustache::context;
import mre::model::to_bytes;

// FIXME: move after https://github.com/mozilla/rust/issues/2242 is fixed.
impl <T: to_mustache> of to_mustache for option<T> {
    fn to_mustache() -> mustache::data {
        alt self {
          none { mustache::bool(false) }
          some(v) { v.to_mustache() }
        }
    }
}

impl of to_mustache for user::user {
    fn to_mustache() -> mustache::data {
        import user::user;

        hash_from_strs([
            ("user_id", self.id())
        ]).to_mustache()
    }
}

impl of to_mustache for post::post {
    fn to_mustache() -> mustache::data {
        import post::post;

        hash_from_strs([
            ("post_id", self.id()),
            ("title", self.title()),
            ("body", self.body())
        ]).to_mustache()
    }
}

impl of to_mustache for comment::comment {
    fn to_mustache() -> mustache::data {
        import comment::comment;

        hash_from_strs([
            ("comment_id", self.id()),
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
        self.reply_http(200u, template)
    }
}

fn login(app: app, username: str, password: str) -> option<user::user> {
    user::find(app.es, "blog", username).chain { |user|
        if app.password_hasher.verify(password, user.password()) {
            some(user)
        } else {
            none
        }
    }
}

fn routes(app: app::app) {
    // Show all the posts.
    app.get("^/$") { |req, rep, _m|
        let posts = post::all(app.es);

        // This can be simplified after mozilla/rust/issues/2258 is fixed.
        rep.render_200(app.mu, "index", hash_from_strs([
            ("user", req.data.user.to_mustache()),
            ("posts", posts.to_mustache())
        ]))
    }

    // Create a user.
    app.get("^/signup$") { |_req, rep, _m|
        let hash: hashmap<str, str> = str_hash();
        rep.render_200(app.mu, "signup", hash)
    }

    app.post("^/signup$") { |req, rep, _m|
        forms::signup(req, rep) { |username, password|
            import user::user;

            let user = user::user(app.es, app.password_hasher, "blog",
                                  username, password);

            alt user.create() {
              ok(()) { rep.reply_redirect("/") }
              err(e) { rep.reply_http(400u, e) }
            }
        }
    }

    // Login a user.
    app.get("^/login$") { |_req, rep, _m|
        let hash: hashmap<str, str> = str_hash();
        rep.render_200(app.mu, "login", hash)
    }

    app.post("^/login$") { |req, rep, _m|
        forms::login(req, rep) { |username, password|
            import user::user;
            alt login(app, username, password) {
              none { rep.reply_http(401u, "") }
              some(user) {
                // Destroy any old sessions.
                alt req.data.session {
                  none {}
                  some(session) {
                    rep.clear_cookie("session");
                    session.delete();
                  }
                }

                // Create a new session.
                let session = session::session(app.es, "blog", user.id());

                alt session.create() {
                  ok(()) {
                    let cookie = cookie::cookie("session", session.id());
                    rep.set_cookie(cookie);
                    rep.reply_redirect("/")
                  }
                  err(e) {
                    rep.reply_http(500u, e)
                  }
                }
              }
            }
        }
    }

    // Logout a user.
    app.post("^/logout$") { |req, rep, _m|
        alt req.data.session {
          none {}
          some(session) {
            session.delete();
            rep.clear_cookie("session");
          }
        }

        rep.reply_redirect("/")
    }

    // Show all the users.
    app.get("^/users$") { |_req, rep, _m|
        let users = user::all(app.es, "blog");

        // This can be simplified after mozilla/rust/issues/2258 is fixed.
        rep.render_200(app.mu, "user_index", hash_from_strs([
            ("users", users)
        ]))
    }

    // Show a user.
    app.get("^/users/(?<id>[-_A-Za-z0-9]+)$") { |_req, rep, m|
        let id = m.named("id");

        alt user::find(app.es, "blog", id) {
          none { rep.reply_http(404u, "") }
          some(user) {
            rep.render_200(app.mu, "user_show", hash_from_strs([
                ("user", user)
            ]))
          }
        }
    }

    // Delete a user.
    app.post("^/users/(?<id>[-_A-Za-z0-9]+)/delete$") { |_req, rep, m|
        let id = m.named("id");

        alt user::find(app.es, "blog", id) {
          none { rep.reply_http(404u, "") }
          some(user) {
            user.delete();

            rep.reply_redirect("/")
          }
        }
    }

    // Create a post.
    app.get("^/posts/new$") { |_req, rep, _m|
        rep.render_200(app.mu, "post_new", post::post(app.es, ""))
    }

    app.post("^/posts$") { |req, rep, _m|
        forms::post(req, rep) { |title, body|
            let post = post::post(app.es, "");

            post.set_title(title);
            post.set_body(body);

            alt post.save() {
              ok(()) { rep.reply_redirect("/posts/" + post.id()) }
              err(e) { rep.reply_http(500u, e) }
            }
        }
    }

    // Show a post.
    app.get("^/posts/(?<id>[-_A-Za-z0-9]+)$") { |_req, rep, m|
        let id = m.named("id");

        alt post::find(app.es, id) {
          none { rep.reply_http(404u, "") }
          some(post) {
            let comments = post.find_comments();

            rep.render_200(app.mu, "post_show", hash_from_strs([
                ("post_id", id.to_mustache()),
                ("post", post.to_mustache()),
                ("comments", comments.to_mustache())
            ]))
          }
        }
    }

    // Edit a post.
    app.get("^/posts/(?<id>[-_A-Za-z0-9]+)/edit$") { |_req, rep, m|
        let id = m.named("id");

        alt post::find(app.es, id) {
          none { rep.reply_http(404u, "") }
          some(post) {
            rep.render_200(app.mu, "post_edit", hash_from_strs([
                ("post_id", id.to_mustache()),
                ("post", post.to_mustache())
            ]))
          }
        }
    }

    app.post("^/posts/(?<id>[-_A-Za-z0-9]+)$") { |req, rep, m|
        let post_id = m.named("id");

        alt post::find(app.es, post_id) {
          none { rep.reply_http(404u, "") }
          some(post) {
            forms::post(req, rep) { |title, body|
                post.set_title(title);
                post.set_body(body);

                alt post.save() {
                  ok(id) { rep.reply_redirect("/posts/" + post_id) }
                  err(e) { rep.reply_http(500u, e) }
                }
            }
          }
        }
    }

    // Delete a post.
    app.post("^/posts/(?<id>[-_A-Za-z0-9]+)/delete$") { |_req, rep, m|
        alt post::find(app.es, m.named("id")) {
          none { rep.reply_http(404u, "") }
          some(post) {
            post.delete();

            rep.reply_redirect("/")
          }
        }
    }

    // Create a comment.
    app.post("^/posts/(?<id>[-_A-Za-z0-9]+)/comments$") { |req, rep, m|
        let id = m.named("id");

        alt post::find(app.es, id) {
          none { rep.reply_http(404u, "") }
          some(post) {
            forms::comment(req, rep) { |body|
                let comment = comment::comment(app.es, id, "");
                comment.set_body(body);

                alt comment.save() {
                  ok(_) { rep.reply_redirect("/posts/" + id) }
                  err(e){ rep.reply_http(500u, e) }
                }
            }
          }
        }
    }

    // Edit a comment.
    app.get("^/posts/(?<post_id>[-_A-Za-z0-9]+)/comments/(?<id>[-_A-Za-z0-9]+)/edit$") { |_req, rep, m|
        let post_id = m.named("post_id");
        let comment_id = m.named("id");

        alt comment::find(app.es, post_id, comment_id) {
          none { rep.reply_http(404u, "") }
          some(comment) {
            rep.render_200(app.mu, "comment_edit", hash_from_strs([
                ("post_id", post_id.to_mustache()),
                ("comment", comment.to_mustache())
            ]))
          }
        }
    }

    app.post("^/posts/(?<post_id>[-_A-Za-z0-9]+)/comments/(?<id>[-_A-Za-z0-9]+)$") { |req, rep, m|
        let post_id = m.named("post_id");
        let comment_id = m.named("id");

        alt comment::find(app.es, post_id, comment_id) {
          none { rep.reply_http(404u, "") }
          some(comment) {
            forms::comment(req, rep) { |body|
                comment.set_body(body);

                alt comment.save() {
                  ok(id) { rep.reply_redirect("/posts/" + post_id) }
                  err(e) { rep.reply_http(500u, e) }
                }
            }
          }
        }
    }

    // Delete a comment.
    app.post("^/posts/(?<post_id>[-_A-Za-z0-9]+)/comments/(?<id>[-_A-Za-z0-9]+)/delete$") { |_req, rep, m|
        let post_id = m.named("post_id");
        let comment_id = m.named("id");

        alt comment::find(app.es, post_id, comment_id) {
          none { rep.reply_http(404u, "") }
          some(comment) {
            comment.delete();

            rep.reply_redirect("/posts/" + post_id)
          }
        }
    }
}
