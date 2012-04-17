import io::{reader, reader_util};
import result::{ok, err, extensions};

import std::map::{hashmap, str_hash, hash_from_strs};
import std::json;

import zmq::{context, error};

import elasticsearch::{client, search_builder, index_builder, json_dict_builder};

import mre::mre;
import mre::response::{response, http_200, http_400, http_404, redirect};

import mongrel2::{connection, request};

import post::post;

fn render_200(req: request, path: str, data: hashmap<str, mustache::data>)
  -> response {
    let template = mustache::render_file(path, data);
    http_200(req, str::bytes(template))
}

fn json_to_mustache(j: json::json) -> mustache::data {
    alt j {
      json::string(s) { mustache::str(s) }
      json::num(n) { mustache::str(#fmt("%f", n)) }
      json::boolean(b) { mustache::bool(b) }
      json::list(v) { mustache::vec(v.map(json_to_mustache)) }
      json::dict(d) {
        let m = str_hash();
        d.items { |k, v| m.insert(k, json_to_mustache(v)); };
        mustache::map(m)
      }
      json::null { mustache::bool(false) }
    }
}

fn main() {
    let ctx =
        alt zmq::init(1) {
          ok(ctx) { ctx }
          err(e) { fail e.to_str() }
        };

    let es = elasticsearch::connect_with_zmq(ctx, "tcp://localhost:9700");

    let m2 = mongrel2::connect(ctx,
        "F0D32575-2ABB-4957-BC8B-12DAC8AFF13A",
        "tcp://127.0.0.1:9998",
        "tcp://127.0.0.1:9999");

    let mre = mre::mre(m2, io::stdout());

    fn post_to_mustache(post: post::post) -> mustache::data {
        mustache::map(hash_from_strs([
            ("post", mustache::map(hash_from_strs([
                ("_id", mustache::str(post.id)),
                ("title", mustache::str(post.title())),
                ("body", mustache::str(post.body()))
            ])))
        ]))
    }

    mre.router.add("GET", "^/$") { |req, _m|
        let posts = post::all(es).map(post_to_mustache);

        render_200(req, "index.mustache", hash_from_strs([
            ("posts", mustache::vec(posts))
        ]))
    }

    mre.router.add("GET", "^/posts/(?<id>\\w+)$") { |req, m|
        alt post::find(es, m.named("id")) {
          none { http_404(req) }
          some(post) {
            let m = alt check post_to_mustache(post) {
              mustache::map(m) { m }
            };

            render_200(req, "show.mustache", m)
          }
        }
    }

    mre.router.add("GET", "^/new-post$") { |req, _m|
        let post = post::post("");

        let m = alt check post_to_mustache(post) {
          mustache::map(m) { m }
        };

        render_200(req, "new.mustache", m)
    }

    mre.router.add("GET", "^/posts/(?<id>\\w+)/edit$") { |req, m|
        alt post::find(es, m.named("id")) {
          none { http_404(req) }
          some(post) {
            let m = alt check post_to_mustache(post) {
              mustache::map(m) { m }
            };

            render_200(req, "edit.mustache", m)
          }
        }
    }

    mre.router.add("POST", "^/posts$") { |req, _m|
        let form = uri::decode_qs(req.body);
        let post = post::post("");

        alt form.find("title") {
          some(title) { post.set_title(title[0]) }
          none {}
        }

        alt form.find("body") {
          some(body) { post.set_body(body[0]) }
          none {}
        }

        alt post.save(es) {
          none { http_400(req) }
          some(id) { redirect(req, "/posts/" + id) }
        }
    }


    mre.router.add("POST", "^/posts/(?<id>\\w+)$") { |req, m|
        let id = m.named("id");
        let form = uri::decode_qs(req.body);

        alt post::find(es, id) {
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

            alt post.save(es) {
              none { http_400(req) }
              some(id) { redirect(req, "/posts/" + id) }
            }
          }
        }
    }

    mre.run();

    m2.term();
    ctx.term();
}
