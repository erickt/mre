import std::map::{hashmap, str_hash};
import std::json;
import elasticsearch::{client, search_builder, index_builder, json_dict_builder};

export post;

type model = {
    _index: str,
    _type: str,
    _id: str,
    _version: option<uint>,
    source: hashmap<str, json::json>,
};

fn model(_index: str, _type: str, _id: str) -> model {
    {
        _index: _index,
        _type: _type,
        _id: _id,
        _version: none,
        source: str_hash()
    }
}

fn hit_to_model(hit: hashmap<str, json::json>) -> model {
    let _index = alt check hit.get("_index") { json::string(s) { s } };
    let _type = alt check hit.get("_type") { json::string(s) { s } };
    let _id = alt check hit.get("_id") { json::string(s) { s } };

    let _version = hit.find("_version").chain { |v|
        alt check v { json::num(n) { some(n as uint) } }
    };

    let source = alt check hit.get("_source") {
      json::dict(source) { source }
    };

    {
        _index: _index,
        _type: _type,
        _id: _id,
        _version: _version,
        source: source
    }
}

fn find_model(es: elasticsearch::client, index: str, typ: str, id: str) -> option<model> {
    let rep = es.get(index, typ, id);

    let hit = alt check rep.body { json::dict(hit) { hit } };

    alt hit.get("exists") {
      json::boolean(true) { some(hit_to_model(hit)) }
      _ { none }
    }
}

fn all_models(es: elasticsearch::client, index: str, typ: str) -> [model] {
    let rep = es.prepare_search()
        .set_index(index)
        .set_type(typ)
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
                        json::dict(hit) { hit_to_model(hit) }
                      }
                  }
              }
            }
          }
        }
      }
    }
}

fn save_model(es: elasticsearch::client, model: model) -> option<str> {
    let index = es.prepare_index(model._index, model._type)
        .set_source(model.source);

    if model._id != "" { index.set_id(model._id); }
    model._version.iter { |v| index.set_version(v); };

    let rep = index.execute();

    if rep.code == 200u || rep.code == 201u {
        let body = alt check rep.body { json::dict(body) { body } };
        alt check body.get("_id") {
          json::string(id) { some(id) }
        }
    } else {
        none
    }
}

fn delete_model(es: elasticsearch::client, model: model) {
    if model._id != "" {
        es.delete(model._index, model._type, model._id);
    }
}

mod post {
    enum t = model;

    fn post(id: str) -> t { t(model("blog", "post", id)) }

    fn find(es: elasticsearch::client, id: str) -> option<t> {
        find_model(es, "blog", "post", id).map { |p| t(p) }
    }

    fn all(es: elasticsearch::client) -> [t] {
        all_models(es, "blog", "post").map { |p| t(p) }
    }
}

impl post for post::t {
    fn title() -> str {
        self.source.find("title").map_default("") { |title|
            alt check title { json::string(title) { title } }
        }
    }

    fn set_title(title: str) {
        self.source.insert("title", json::string(title));
    }

    fn body() -> str {
        self.source.find("body").map_default("") { |body|
            alt check body { json::string(body) { body } }
        }
    }

    fn set_body(body: str) {
        self.source.insert("body", json::string(body));
    }

    fn save(es: elasticsearch::client) -> option<str> {
        save_model(es, *self)
    }

    fn delete(es: elasticsearch::client) {
        delete_model(es, *self)
    }
}
