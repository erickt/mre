import elasticsearch::{
    client,
    json_dict_builder,
    search_builder,
    index_builder,
    delete_builder
};

export model;
export find;
export search;
export all;

type model = {
    es: client,
    _index: str,
    _type: str,
    _id: str,
    _parent: option<str>,
    _version: option<uint>,
    source: hashmap<str, json>,
};

fn model(es: client, index: str, typ: str, id: str) -> model {
    {
        es: es,
        _index: index,
        _type: typ,
        _id: id,
        _parent: none,
        _version: none,
        source: str_hash()
    }
}

fn hit_to_model(es: client, hit: hashmap<str, json::json>) -> model {
    let index = alt check hit.get("_index") { json::string(s) { s } };
    let typ = alt check hit.get("_type") { json::string(s) { s } };
    let id = alt check hit.get("_id") { json::string(s) { s } };

    let parent = hit.find("_parent").chain { |s|
        alt check s { json::string(s) { some(s) } }
    };

    let version = hit.find("_version").chain { |v|
        alt check v { json::num(n) { some(n as uint) } }
    };

    let source = alt check hit.get("_source") {
      json::dict(source) { source }
    };

    {
        es: es,
        _index: index,
        _type: typ,
        _id: id,
        _parent: parent,
        _version: version,
        source: source
    }
}

fn find(es: client, index: str, typ: str, id: str) -> option<model> {
    let rep = es.get(index, typ, id);

    // Fail if ES had an error.
    if rep.code != 200u {
        fail json::to_str(rep.body);
    }

    let hit = alt check rep.body { json::dict(hit) { hit } };

    alt hit.get("exists") {
      json::boolean(true) { some(hit_to_model(es, hit)) }
      _ { none }
    }
}

fn search(es: client, f: fn(search_builder)) -> [model] {
    let bld = es.prepare_search();
    f(bld);

    let rep = bld.execute();

    // Fail if ES had an error.
    if rep.code != 200u {
        fail json::to_str(rep.body);
    }

    alt check rep.body {
      json::dict(body) {
        alt check body.get("hits") {
          json::dict(hits) {
            alt check hits.get("hits") {
              json::list(hits) {
                  hits.map { |hit|
                      alt check hit {
                        json::dict(hit) { hit_to_model(es, hit) }
                      }
                  }
              }
            }
          }
        }
      }
    }
}

fn all(es: client, index: str, typ: str) -> [model] {
    search(es) { |bld|
        bld
            .set_indices([index])
            .set_types([typ])
            .set_source(*json_dict_builder()
                .insert_dict("query") { |bld|
                    bld.insert_dict("match_all") { |_bld| };
                }
            );
    }
}

impl model for model {
    fn set_null(key: str) -> bool {
        self.source.insert(key, json::null)
    }

    fn get_bool(key: str) -> bool {
        alt self.find_bool(key) {
          none { fail }
          some(value) { value }
        }
    }

    fn find_bool(key: str) -> option<bool> {
        self.source.find(key).chain { |value|
            alt value {
              json::boolean(value) { some(value) }
              _ { none }
            }
        }
    }

    fn set_bool(key: str, value: bool) -> bool {
        self.source.insert(key, json::boolean(value))
    }

    fn find_str(key: str) -> option<str> {
        self.source.find(key).chain { |value|
            alt value {
              json::string(value) { some(value) }
              _ { none }
            }
        }
    }

    fn get_str(key: str) -> str {
        alt self.find_str(key) {
          none { fail }
          some(value) { value }
        }
    }

    fn set_str(key: str, value: str) -> bool {
        self.source.insert(key, json::string(value))
    }

    fn find_float(key: str) -> option<float> {
        self.source.find(key).chain { |value|
            alt value {
              json::num(value) { some(value) }
              _ { none }
            }
        }
    }

    fn get_float(key: str) -> float {
        alt self.find_float(key) {
          none { fail }
          some(value) { value }
        }
    }

    fn set_float(key: str, value: float) -> bool {
        self.source.insert(key, json::num(value))
    }

    fn find_uint(key: str) -> option<uint> {
        self.find_float(key).map { |value| value as uint }
    }

    fn get_uint(key: str) -> uint {
        self.get_float(key) as uint
    }

    fn set_uint(key: str, value: uint) -> bool {
        self.set_float(key, value as float)
    }

    fn find_int(key: str) -> option<int> {
        self.find_float(key).map { |value| value as int }
    }

    fn get_int(key: str) -> int {
        self.get_float(key) as int
    }

    fn set_int(key: str, value: int) -> bool {
        self.set_float(key, value as float)
    }

    fn index(op_type: elasticsearch::op_type) -> result<(str, uint), str> {
        let index = self.es.prepare_index(self._index, self._type)
            .set_op_type(op_type)
            .set_source(self.source)
            .set_refresh(true);

        if self._id != "" { index.set_id(self._id); }
        self._parent.iter { |p| index.set_parent(p); }
        self._version.iter { |v| index.set_version(v); }

        let rep = index.execute();

        if rep.code == 200u || rep.code == 201u {
            let body = alt check rep.body { json::dict(body) { body } };
            let id = alt check body.get("_id") {
              json::string(id) { id }
            };
            let version = alt check body.get("_version") {
              json::num(version) { version as uint }
            };

            ok((id, version))

        } else {
            let body = alt check rep.body { json::dict(body) { body } };
            let e = alt check body.get("error") { json::string(e) { e } };
            err(e)
        }
    }

    fn create() -> result<(str, uint), str> {
        self.index(elasticsearch::CREATE)
    }

    fn save() -> result<(str, uint), str> {
        self.index(elasticsearch::INDEX)
    }

    fn delete() {
        if self._id != "" {
            self.es.prepare_delete(self._index, self._type, self._id)
                .set_refresh(true)
                .execute();
        }
    }
}
