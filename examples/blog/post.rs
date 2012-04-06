import std::map::{hashmap, str_hash};
import std::json;
import elasticsearch;

type post = {
    id: str,
    version: option<uint>,
    source: hashmap<str, json::json>,
};

fn post(id: str) -> post {
    { id: id, version: none, source: str_hash() }
}

fn hit_to_post(hit: hashmap<str, json::json>) -> post {
    let id = alt check hit.get("_id") { json::string(id) { id } };

    let version = hit.find("_version").chain { |v|
        alt check v { json::num(n) { some(n as uint) } }
    };

    let source = alt check hit.get("_source") {
      json::dict(source) { source }
    };

    { id: id, version: version, source: source }
}

fn find(es: elasticsearch::client, id: str) -> option<post> {
    let rep = es.get("blog", "post", id);

    let hit = alt check rep.body { json::dict(hit) { hit } };

    alt hit.get("exists") {
      json::boolean(true) { some(hit_to_post(hit)) }
      _ { none }
    }
}

fn all(es: elasticsearch::client) -> [post] {
    let rep = es.prepare_search()
        .set_index("blog")
        .set_source(json_dict_builder()
            .insert_dict("query") { |bld|
                bld.insert_dict("match_all") { |_bld| };
            }
        ).execute();

    alt check rep.body {
      json::dict(body) {
        alt check body.get("hits") {
          json::dict(hits) {
            alt check hits.get("hits") {
              json::list(hits) {
                  hits.map { |hit|
                      alt check hit {
                        json::dict(hit) { hit_to_post(hit) }
                      }
                  }
              }
            }
          }
        }
      }
    }
}

impl post for post {
    fn title() -> str {
        self.source.find("title").with_option("", { |title|
            alt check title { json::string(title) { title } }
        })
    }

    fn set_title(title: str) {
        self.source.insert("title", json::string(title));
    }

    fn body() -> str {
        self.source.find("body").with_option("") { |body|
            alt check body { json::string(body) { body } }
        }
    }

    fn set_body(body: str) {
        self.source.insert("body", json::string(body));
    }

    fn save(es: elasticsearch::client) -> option<str> {
        let index = es.prepare_index("blog", "post")
            .set_source(self.source);

        if self.id != "" { index.set_id(self.id); }
        self.version.with_option_do { |v| index.set_version(v); };

        let rep = index.execute();

        #error("%?", rep);

        if rep.code == 200u {
            let body = alt check rep.body { json::dict(body) { body } };
            alt check body.get("_id") {
              json::string(id) { some(id) }
            }
        } else {
            none
        }
    }
}
