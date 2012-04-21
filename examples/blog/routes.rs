import mre::response::{
    response,
    http_200,
    http_400,
    http_404,
    http_500,
    redirect
};
import mustache::context;

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
    // Show all the posts.
    app.get("^/$") { |req, _m|
        let posts = models::post::all(app.es);

        // This can be simplified after mozilla/rust/issues/2258 is fixed.
        app.render_200(req, "index", hash_from_strs([
            ("posts", posts.to_mustache())
        ]).to_mustache())
    }

    // Create a post.
    app.get("^/posts/new$") { |req, _m|
        app.render_200(req, "post_new", post::post(""))
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
          ok(id) { redirect(req, "/posts/" + id) }
          err(e) { http_500(req, str::bytes(e)) }
        }
    }

    // Show a post.
    app.get("^/posts/(?<id>[-_A-Za-z0-9]+)$") { |req, m|
        let id = m.named("id");

        alt models::post::find(app.es, id) {
          none { http_404(req) }
          some(post) {
            let comments = post.find_comments(app.es);

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

        alt models::post::find(app.es, id) {
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

        alt models::post::find(app.es, post_id) {
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
              ok(id) { redirect(req, "/posts/" + post_id) }
              err(e) { http_500(req, str::bytes(e)) }
            }
          }
        }
    }

    // Delete a post.
    app.post("^/posts/(?<id>[-_A-Za-z0-9]+)/delete$") { |req, m|
        alt models::post::find(app.es, m.named("id")) {
          none { http_404(req) }
          some(post) {
            post.delete(app.es);

            redirect(req, "/")
          }
        }
    }

    // Create a comment.
    app.post("^/posts/(?<id>[-_A-Za-z0-9]+)/comments$") { |req, m|
        let id = m.named("id");
        let form = uri::decode_qs(req.body);

        alt models::post::find(app.es, id) {
          none { http_404(req) }
          some(post) {
            let comment = models::comment::comment(id, "");

            alt form.find("body") {
              some(body) { comment.set_body(body[0]) }
              none {}
            }

            alt comment.save(app.es) {
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

        alt models::comment::find(app.es, post_id, comment_id) {
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

        alt models::comment::find(app.es, post_id, comment_id) {
          none { http_404(req) }
          some(comment) {
            alt form.find("body") {
              some(body) { comment.set_body(body[0]) }
              none {}
            }

            alt comment.save(app.es) {
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

        alt models::comment::find(app.es, post_id, comment_id) {
          none { http_404(req) }
          some(comment) {
            comment.delete(app.es);

            redirect(req, "/posts/" + post_id)
          }
        }
    }
}
