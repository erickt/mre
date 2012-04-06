import std::map;
import std::map::hashmap;

type request = mongrel2::request::t;

fn from_req(req: mongrel2::request::t) -> request {
    req
}

fn empty_request() -> request {
    {
        uuid: "",
        id: "",
        path: "",
        headers: map::str_hash(),
        body: []
    }
}

fn builder() -> @mut request {
    @mut empty_request()
}

impl builder for @mut request {
    fn set_uuid(uuid: str) -> @mut request {
        *self = { uuid: uuid with *self };
        self
    }

    fn set_id(id: str) -> @mut request {
        *self = { id: id with *self };
        self
    }

    fn set_path(path: str) -> @mut request {
        *self = { path: path with *self };
        self
    }

    fn add_header(key: str, value: str) -> @mut request {
        let values = alt self.headers.find(key) {
          none { [] }
          some(values) { values }
        };
        self.headers.insert(key, values + [value]);

        self
    }
}
