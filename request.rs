export method;
export request;

enum method {
    HEAD,
    GET,
    POST,
    PUT,
    DELETE,
    TRACE,
    OPTIONS,
    CONNECT,
    PATCH
}

type request<T> = {
    req: @mongrel2::request,
    method: method,
    cookies: hashmap<str, cookie::cookie>,
    mut data: T,
};

fn request<T: copy>(req: @mongrel2::request, rep: @response, data: T)
  -> option<@request<T>> {
    let method = alt req.headers.find("METHOD") {
      none {
        rep.http_400("");
        ret none
      }
      some(methods) {
        alt methods[0] {
          "HEAD" { HEAD }
          "GET" { GET }
          "POST" { POST }
          "PUT" { PUT }
          "DELETE" { DELETE }
          "TRACE" { TRACE }
          "OPTIONS" { OPTIONS }
          "CONNECT" { CONNECT }
          "PATCH" { PATCH }
          _ {
            rep.http_501("");
            ret none
          }
        }
      }
    };

    let cookies = alt req.headers.find("cookie") {
      none { str_hash() }
      some(cookies) {
        alt cookie::parse_headers(cookies) {
          ok(cookies) { cookies }
          err(e) {
            rep.http_400(e);
            ret none
          }
        }
      }
    };

    some(@{
        req: req,
        method: method,
        cookies: cookies,
        mut data: data,
    })
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
