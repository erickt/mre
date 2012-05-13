type request<T> = {
    req: mongrel2::request,
    cookies: hashmap<str, cookie::cookie>,
    mut data: T,
};

fn request<T: copy>(req: mongrel2::request,
                    data: T) -> result<@request<T>, str> {
    let cookies = alt req.headers.find("cookie") {
      none { str_hash() }
      some(cookies) {
        alt cookie::parse_headers(cookies) {
          ok(cookies) { cookies }
          err(e) { ret err(e); }
        }
      }
    };

    ok(@{ req: req, cookies: cookies, mut data: data })
}

impl request<T: copy> for @request<T> {
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
