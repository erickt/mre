use elasticsearch::{
    Client,
    JsonObjectBuilder,
    SearchBuilder,
};

pub struct Error {
    code: uint,
    msg: ~str,
}

impl Error: to_str::ToStr {
    pure fn to_str() -> ~str { fmt!("[%u] %s", self.code, self.msg) }
}

impl Error: ToBytes {
    fn to_bytes(lsb: bool) -> ~[u8] { self.to_str().to_bytes(lsb) }
}

pub struct Model {
    es: Client,
    _index: ~str,
    _type: ~str,
    mut _id: ~str,
    _parent: Option<~str>,
    mut _version: Option<uint>,
    source: json::Object,
}

pub fn Model(es: Client, index: ~str, typ: ~str, id: ~str) -> Model {
    Model {
        es: es,
        _index: index,
        _type: typ,
        _id: id,
        _parent: None,
        _version: None,
        source: LinearMap()
    }
}

fn hit_to_model(es: Client, hit: ~json::Object) -> Model {
    let mut hit = move hit;

    let index = match hit.pop(&~"_index") {
        None => fail,
        Some(json::String(move s)) => s,
        Some(_) => fail,
    };

    let typ = match hit.pop(&~"_type") {
        None => fail,
        Some(json::String(move s)) => s,
        Some(_) => fail,
    };

    let id = match hit.pop(&~"_id") {
        None => fail,
        Some(json::String(move s)) => s,
        Some(_) => fail,
    };

    let parent = match hit.pop(&~"_parent") {
        None => None,
        Some(json::String(move s)) => Some(s),
        Some(_) => fail,
    };

    let version = match hit.pop(&~"_version") {
        None => None,
        Some(json::Number(n)) => Some(n as uint),
        Some(_) => fail,
    };

    let source = match hit.pop(&~"_source") {
        None => fail,
        Some(json::Object(move source)) => source,
        Some(_) => fail,
    };

    Model {
        es: es,
        _index: index,
        _type: typ,
        mut _id: id,
        mut _parent: parent,
        mut _version: version,
        source: *source
    }
}

pub fn find(es: Client, index: ~str, typ: ~str, id: ~str) -> Option<Model> {
    let mut rep = es.get(index, typ, id);

    // Fail if ES had an error.
    if rep.code != 200u { fail rep.body.to_str(); }

    // FIXME: https://github.com/mozilla/rust/issues/3767
    let hit = match rep.body {
        json::Object(hit) => copy hit,
        _ => fail
    };

    match hit.get_ref(&~"exists") {
      &json::Boolean(true) => {},
      _ => return None,
    }

    Some(hit_to_model(es, move hit))
}

pub fn search(es: Client, f: fn(@SearchBuilder)) -> ~[Model] {
    let bld = es.prepare_search();
    f(bld);

    let rep = bld.execute();

    // Fail if ES had an error.
    if rep.code != 200u { fail rep.body.to_str(); }

    // FIXME: https://github.com/mozilla/rust/issues/3767
    match rep.body {
        json::Object(body) => {
            match body.get_ref(&~"hits") {
                &json::Object(hits) => {
                    match hits.get_ref(&~"hits") {
                        &json::List(hits) => {
                            do hits.map |hit| {
                                match hit {
                                    &json::Object(hit) => {
                                        hit_to_model(es, copy hit)
                                    }
                                    _ => fail
                                }
                            }
                        },
                        _ => fail
                    }
                },
                _ => fail
            }
        },
      _ => fail
    }
}

pub fn all(es: Client, index: ~str, typ: ~str) -> ~[Model] {
    do search(es) |bld| {
        bld
            .set_indices(~[copy index])
            .set_types(~[copy typ])
            .set_source(
                JsonObjectBuilder()
                    .insert_object(~"query", |bld| {
                        bld.insert_object(~"match_all", |_bld| { });
                    })
                    .object.take()
            );
    }
}

impl Model {
    fn set_null(&mut self, key: ~str) -> bool {
        self.source.insert(key, json::Null)
    }

    fn get_bool(&self, key: &~str) -> bool {
        match self.find_bool(key) {
            None => fail,
            Some(value) => value,
        }
    }

    fn find_bool(&self, key: &~str) -> Option<bool> {
        match self.source.find_ref(key) {
            None => None,
            Some(&json::Boolean(value)) => Some(value),
            Some(_) => None,
        }
    }

    fn set_bool(&mut self, key: ~str, value: bool) -> bool {
        self.source.insert(key, json::Boolean(value))
    }

    fn find_str(&self, key: &~str) -> Option<~str> {
        match self.source.find_ref(key) {
            None => None,
            Some(&json::String(value)) => Some(copy value),
            Some(_) => None,
        }
    }

    fn get_str(&self, key: &~str) -> ~str {
        match self.find_str(key) {
            None => fail,
            Some(value) => copy value,
        }
    }

    fn set_str(&mut self, key: ~str, value: ~str) -> bool {
        self.source.insert(key, json::String(value))
    }

    fn find_float(&self, key: &~str) -> Option<float> {
        match self.source.find_ref(key) {
            None => None,
            Some(&json::Number(value)) => Some(value),
            Some(_) => None,
        }
    }

    fn get_float(&self, key: &~str) -> float {
        match self.find_float(key) {
            None => fail,
            Some(value) => value,
        }
    }

    fn set_float(&mut self, key: ~str, value: float) -> bool {
        self.source.insert(key, json::Number(value))
    }

    fn find_uint(&self, key: &~str) -> Option<uint> {
        self.find_float(key).map(|value| *value as uint)
    }

    fn get_uint(&self, key: &~str) -> uint {
        self.get_float(key) as uint
    }

    fn set_uint(&mut self, key: ~str, value: uint) -> bool {
        self.set_float(key, value as float)
    }

    fn find_int(&self, key: &~str) -> Option<int> {
        self.find_float(key).map(|value| *value as int)
    }

    fn get_int(&self, key: &~str) -> int {
        self.get_float(key) as int
    }

    fn set_int(&mut self, key: ~str, value: int) -> bool {
        self.set_float(key, value as float)
    }

    fn index(&self, op_type: elasticsearch::OpType) -> Result<(), Error> {
        let index = self.es.prepare_index(copy self._index, copy self._type)
            .set_op_type(op_type)
            .set_source(~copy self.source)
            .set_refresh(true);

        if !self._id.is_empty() { index.set_id(copy self._id); }

        match self._parent {
            None => {},
            Some(copy parent) => { index.set_parent(parent); },
        }

        match self._version {
            None => {},
            Some(copy version) => { index.set_version(version); },
        }

        let rep = index.execute();

        if rep.code == 200 || rep.code == 201 {
            let body = match rep.body {
                json::Object(ref body) => body,
                _ => fail,
            };

            let id = match body.get_ref(&~"_id") {
                &json::String(id) => copy id,
                _ => fail,
            };
            let version = match body.get_ref(&~"_version") {
                &json::Number(version) => version as uint,
                _ => fail,
            };

            // Update our id and version.
            self._id = id;
            self._version = Some(version);

            Ok(())

        } else {
            match rep.body {
                json::Object(ref body) => {
                    match body.get_ref(&~"error") {
                        &json::String(ref e) => {
                            Err(Error { code: rep.code, msg: *e })
                        },
                        _ => fail,
                    }
                }
                _ => fail,
            }
        }
    }

    fn create(&self) -> Result<(), Error> {
        self.index(elasticsearch::CREATE)
    }

    fn save(&self) -> Result<(), Error> {
        self.index(elasticsearch::INDEX)
    }

    fn delete(&self) {
        if !self._id.is_empty() {
            self.es.prepare_delete(
                    copy self._index,
                    copy self._type,
                    copy self._id)
                .set_refresh(true)
                .execute();
        }
    }
}
