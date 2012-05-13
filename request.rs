type request = {
    req: mongrel2::request,
};

fn request(req: mongrel2::request) -> @request {
    @{ req: req }
}

impl request for @request {
    fn body() -> [u8] {
        self.req.body
    }

    fn path() -> str {
        self.req.path
    }

    fn is_disconnect() -> bool {
        import mongrel2::request;
        self.req.is_disconnect()
    }

    fn find_headers(key: str) -> option<[str]> {
        self.req.headers.find(key)
    }

    fn find_header(key: str) -> option<str> {
        self.find_headers(key).chain { |values|
            if values.len() == 0u {
                none
            } else {
                some(values[0])
            }
        }
    }
}
