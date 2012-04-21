import elasticsearch::{
    client,
    index_builder,
    search_builder,
    delete_builder,
    delete_by_query_builder,
    json_dict_builder
};

export post;
export comment;

type model = {
    _index: str,
    _type: str,
    _id: str,
    _parent: option<str>,
    _version: option<uint>,
    source: hashmap<str, json::json>,
};

fn model(_index: str, _type: str, _id: str) -> model {
    {
        _index: _index,
        _type: _type,
        _id: _id,
        _parent: none,
        _version: none,
        source: str_hash()
    }
}

fn hit_to_model(hit: hashmap<str, json::json>) -> model {
    let _index = alt check hit.get("_index") { json::string(s) { s } };
    let _type = alt check hit.get("_type") { json::string(s) { s } };
    let _id = alt check hit.get("_id") { json::string(s) { s } };

    let _parent = hit.find("_parent").chain { |s|
        alt check s { json::string(s) { some(s) } }
    };

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
        _parent: _parent,
        _version: _version,
        source: source
    }
}

fn rep_to_models(rep: elasticsearch::response) -> [model] {
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

fn find_model(es: client, index: str, typ: str, id: str) -> option<model> {
    let rep = es.get(index, typ, id);

    let hit = alt check rep.body { json::dict(hit) { hit } };

    alt hit.get("exists") {
      json::boolean(true) { some(hit_to_model(hit)) }
      _ { none }
    }
}

fn all_models(es: client, index: str, typ: str) -> [model] {
    let rep = es.prepare_search()
        .set_indices([index])
        .set_types([typ])
        .set_source(json_dict_builder()
            .insert_dict("query") { |bld|
                bld.insert_dict("match_all") { |_bld| };
            }
        ).execute();

    rep_to_models(rep)
}

fn save_model(es: client, model: model) -> result<str, str> {
    let index = es.prepare_index(model._index, model._type)
        .set_source(model.source)
        .set_refresh(true);

    if model._id != "" { index.set_id(model._id); }
    model._parent.iter { |p| index.set_parent(p); }
    model._version.iter { |v| index.set_version(v); }

    let rep = index.execute();

    if rep.code == 200u || rep.code == 201u {
        let body = alt check rep.body { json::dict(body) { body } };
        alt check body.get("_id") {
          json::string(id) { ok(id) }
        }
    } else {
        let body = alt check rep.body { json::dict(body) { body } };
        let e = alt check body.get("error") { json::string(e) { e } };
        err(e)
    }
}

mod post {
    enum t = model;

    fn post(id: str) -> t { t(model("blog", "post", id)) }

    fn find(es: client, id: str) -> option<t> {
        find_model(es, "blog", "post", id).map { |p| t(p) }
    }

    fn all(es: client) -> [t] {
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

    fn save(es: client) -> result<str, str> {
        save_model(es, *self)
    }

    fn delete(es: client) {
        if self._id != "" {
            comment::delete_by_post(es, self._id);
            es.prepare_delete(self._index, self._type, self._id)
                .set_refresh(true)
                .execute();
        }
    }

    fn find_comments(es: client) -> [comment::t] {
        comment::find_by_post(es, self._id)
    }
}

mod comment {
    enum t = model;

    fn comment(post_id: str, id: str) -> t {
        t({ _parent: some(post_id) with model("blog", "comment", id) })
    }

    fn find(es: client, post_id: str, id: str) -> option<t> {
        find_model(es, "blog", "comment", id).map { |model|
            t({ _parent: some(post_id) with model })
        }
    }

    fn find_by_post(es: client, post_id: str) -> [t] {
        let rep = es.prepare_search()
            .set_indices(["blog"])
            .set_types(["comment"])
            .set_source(json_dict_builder()
                .insert_dict("filter") { |bld|
                    bld.insert_dict("term") { |bld|
                        bld.insert_str("_parent", post_id);
                    };
                }
            ).execute();

        rep_to_models(rep).map { |model|
            t({ _parent: some(post_id) with model })
        }
    }

    fn delete_by_post(es: client, post_id: str) {
        let rep = es.prepare_delete_by_query()
            .set_indices(["blog"])
            .set_types(["comment"])
            .set_source(json_dict_builder()
                .insert_dict("term") { |bld|
                    bld.insert_str("_parent", post_id);
                }
            ).execute();

        #error("%?", rep);
    }
}

impl comment for comment::t {
    fn body() -> str {
        self.source.find("body").map_default("") { |body|
            alt check body { json::string(body) { body } }
        }
    }

    fn set_body(body: str) {
        self.source.insert("body", json::string(body));
    }

    fn save(es: client) -> result<str, str> {
        save_model(es, *self)
    }

    fn delete(es: client) {
        if self._id != "" {
            es.prepare_delete(self._index, self._type, self._id)
                .set_refresh(true)
                .execute();
        }
    }
}
