import models::{post, comment};

// FIXME: move after https://github.com/mozilla/rust/issues/2242 is fixed.
impl of to_mustache for models::post::t {
    fn to_mustache() -> mustache::data {
        hash_from_strs([
            ("_id", self._id),
            ("title", self.title()),
            ("body", self.body())
        ]).to_mustache()
    }
}

impl of to_mustache for models::comment::t {
    fn to_mustache() -> mustache::data {
        hash_from_strs([
            ("_id", self._id),
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
    app.get("^/$") { |req, _m|
        let posts = models::post::all(app.es);

        // This can be simplified after mozilla/rust/issues/2258 is fixed.
        app.render_200(req, "index", hash_from_strs([
            ("posts", posts.to_mustache())
        ]).to_mustache())
    }

    app.get("^/posts/new$") { |req, _m|
        app.render_200(req, "new", post::post(""))
    }

    app.get("^/posts/(?<id>[-_A-Za-z0-9]+)$") { |req, m|
        alt models::post::find(app.es, m.named("id")) {
          none { http_404(req) }
          some(post) { app.render_200(req, "show", post) }
        }
    }

    app.get("^/posts/(?<id>[-_A-Za-z0-9]+)/edit$") { |req, m|
        alt models::post::find(app.es, m.named("id")) {
          none { http_404(req) }
          some(post) { app.render_200(req, "edit", post) }
        }
    }

    app.post("^/posts$") { |req, _m|
        let form = uri::decode_qs(req.body);
        let post = models::post::post("");

        alt form.find("title") {
          some(title) { post.set_title(title[0]) }
          none {}
        }

        alt form.find("body") {
          some(body) { post.set_body(body[0]) }
          none {}
        }

        alt post.save(app.es) {
          none { http_400(req) }
          some(id) { redirect(req, "/posts/" + id) }
        }
    }

    app.post("^/posts/(?<id>[-_A-Za-z0-9]+)$") { |req, m|
        let id = m.named("id");
        let form = uri::decode_qs(req.body);

        alt models::post::find(app.es, id) {
          none { http_404(req) }
          some(post) {
            alt form.find("title") {
              some(title) { post.set_title(title[0]) }
              none {}
            }

            alt form.find("body") {
              some(body) { post.set_body(body[0]) }
              none {}
            }

            alt post.save(app.es) {
              none { http_400(req) }
              some(id) { redirect(req, "/posts/" + id) }
            }
          }
        }
    }

    app.post("^/posts/(?<id>[-_A-Za-z0-9]+)/delete$") { |req, m|
        alt models::post::find(app.es, m.named("id")) {
          none { http_404(req) }
          some(post) {
            post.delete(app.es);

            redirect(req, "/")
          }
        }
    }
}
