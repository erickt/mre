use elasticsearch::{
    Client,
    JsonObjectBuilder,
    SearchBuilder,
};

pub type error = {
    code: uint,
    msg: @str
};

impl error: to_str::ToStr {
    fn to_str() -> str { #fmt("[%u] %s", self.code, *self.msg) }
}

impl error: ToBytes {
    fn to_bytes() -> ~[u8] { self.to_str().to_bytes() }
}

pub type model = @{
    es: Client,
    _index: @str,
    _type: @str,
    mut _id: @str,
    mut _parent: Option<@str>,
    mut _version: Option<uint>,
    source: HashMap<str, Json>,
};

pub fn model(es: Client, index: @str, typ: @str, id: @str) -> model {
    @{
        es: es,
        _index: index,
        _type: typ,
        mut _id: id,
        mut _parent: None,
        mut _version: None,
        source: HashMap()
    }
}

fn hit_to_model(es: Client, hit: HashMap<str, json::Json>) -> model {
    let index = match hit.get("_index") { json::String(s) => s, _ => fail };
    let typ = match hit.get("_type") { json::String(s) => s, _ => fail };
    let id = match hit.get("_id") { json::String(s) => s, _ => fail };

    let parent = do hit.find("_parent").chain |s| {
        match s { json::String(s) => Some(s), _ => fail }
    };

    let version = do hit.find("_version").chain |v| {
        match v { json::Number(n) => Some(n as uint), _ => fail }
    };

    let source = match hit.get("_source") {
      json::Object(source) => source,
      _ => fail
    };

    @{
        es: es,
        _index: index,
        _type: typ,
        mut _id: id,
        mut _parent: parent,
        mut _version: version,
        source: source
    }
}

pub fn find(es: Client, index: @str, typ: @str, id: @str) -> Option<model> {
    let rep = es.get(*index, copy *typ, copy *id);

    // Fail if ES had an error.
    if rep.code != 200u {
        fail json::to_str(rep.body);
    }

    let hit = match rep.body { json::Object(hit) => hit, _ => fail };

    match hit.get("exists") {
      json::Boolean(true) => Some(hit_to_model(es, hit)),
      _ => None,
    }
}

pub fn search(es: Client, f: fn(SearchBuilder)) -> ~[model] {
    let bld = es.prepare_search();
    f(bld);

    let rep = bld.execute();

    // Fail if ES had an error.
    if rep.code != 200u {
        fail json::to_str(rep.body);
    }

    match rep.body {
        json::Object(body) => {
            match body.get("hits") {
                json::Object(hits) => {
                    match hits.get("hits") {
                        json::List(hits) => {
                            do (*hits).map |hit| {
                                match hit {
                                    json::Object(hit) => hit_to_model(es, hit),
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

pub fn all(es: Client, index: @str, typ: @str) -> ~[model] {
    do search(es) |bld| {
        bld
            .set_indices(~[copy *index])
            .set_types(~[copy *typ])
            .set_source(*JsonObjectBuilder()
                .insert_dict("query", |bld| {
                    bld.insert_dict("match_all", |_bld| { });
                }
                .take())
            );
    }
}

impl model {
    fn set_null(+key: str) -> bool {
        self.source.insert(key, json::Null)
    }

    fn get_bool(+key: str) -> bool {
        match self.find_bool(key) {
            None => fail,
            Some(value) => value,
        }
    }

    fn find_bool(+key: str) -> Option<bool> {
        do self.source.find(key).chain |value| {
            match value {
                json::Boolean(value) => Some(value),
                _ => None,
            }
        }
    }

    fn set_bool(+key: str, value: bool) -> bool {
        self.source.insert(key, json::Boolean(value))
    }

    fn find_str(+key: str) -> Option<@str> {
        do self.source.find(key).chain |value| {
            match value {
                json::String(value) => Some(value),
                _ => None,
            }
        }
    }

    fn get_str(+key: str) -> @str {
        match self.find_str(key) {
            None => fail,
            Some(value) => value,
        }
    }

    fn set_str(+key: str, value: @str) -> bool {
        self.source.insert(key, json::String(value))
    }

    fn find_float(+key: str) -> Option<float> {
        do self.source.find(key).chain |value| {
            match value {
                json::Number(value) => Some(value),
                _ => None,
            }
        }
    }

    fn get_float(+key: str) -> float {
        match self.find_float(key) {
            None => fail,
            Some(value) => value,
        }
    }

    fn set_float(+key: str, value: float) -> bool {
        self.source.insert(key, json::Number(value))
    }

    fn find_uint(+key: str) -> Option<uint> {
        self.find_float(key).map(|value| value as uint)
    }

    fn get_uint(+key: str) -> uint {
        self.get_float(key) as uint
    }

    fn set_uint(+key: str, value: uint) -> bool {
        self.set_float(key, value as float)
    }

    fn find_int(+key: str) -> Option<int> {
        self.find_float(key).map(|value| value as int)
    }

    fn get_int(+key: str) -> int {
        self.get_float(key) as int
    }

    fn set_int(+key: str, value: int) -> bool {
        self.set_float(key, value as float)
    }

    fn index(op_type: elasticsearch::OpType) -> Result<(), error> {
        let index = self.es.prepare_index(copy *self._index, copy *self._type)
            .set_op_type(op_type)
            .set_source(self.source)
            .set_refresh(true);

        if *self._id != "" { index.set_id(copy *self._id); }

        (copy self._parent).iter(|p| { index.set_parent(copy *p); });
        (copy self._version).iter(|v| { index.set_version(v); });

        let rep = index.execute();

        if rep.code == 200u || rep.code == 201u {
            let body = match rep.body { json::Object(body) => body, _ => fail };
            let id = match body.get("_id") {
                json::String(id) => id,
                _ => fail,
            };
            let version = match body.get("_version") {
                json::Number(version) => version as uint,
                _ => fail,
            };

            // Update our id and version.
            self._id = id;
            self._version = Some(version);

            Ok(())

        } else {
            let body = match rep.body { json::Object(body) => body, _ => fail };
            let e = match body.get("error") { json::String(e) => e, _ => fail };
            Err({ code: rep.code, msg: e })
        }
    }

    fn create() -> Result<(), error> {
        self.index(elasticsearch::CREATE)
    }

    fn save() -> Result<(), error> {
        self.index(elasticsearch::INDEX)
    }

    fn delete() {
        if *self._id != "" {
            self.es.prepare_delete(
                    copy *self._index,
                    copy *self._type,
                    copy *self._id)
                .set_refresh(true)
                .execute();
        }
    }
}
