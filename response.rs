import cookie::cookie;
import to_bytes::to_bytes;

type response = {
    mut code: uint,
    mut status: str,
    headers: hashmap<str, [str]>,
    mut reply: fn@([u8]),
    mut end: fn@(),
};

fn response(m2: mongrel2::connection, req: @mongrel2::request) -> @response {
    @{
        mut code: 200u,
        mut status: "OK",
        headers: str_hash(),
        mut reply: { |msg| m2.reply(req, msg); },
        mut end: { || m2.reply(req, str::bytes("")); },
    }
}

impl response for @response {
    fn set_status(code: uint, status: str) {
        self.code = code;
        self.status = status;
    }

    fn find_headers(key: str) -> option<[str]> {
        self.headers.find(key)
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

    fn set_header(name: str, value: str) {
        let mut values = alt self.headers.find(name) {
          none { [] }
          some(values) { values }
        };

        vec::push(values, value);

        self.headers.insert(name, values);
    }

    fn set_cookie(cookie: cookie::cookie) {
        let header = alt cookie.to_header() {
          ok(header) { header }
          err(e) { fail e; }
        };

        self.set_header("Set-Cookie", header);
    }

    fn clear_cookie(name: str) {
        let cookie = cookie::cookie(name, "");
        cookie.max_age = some(0u);
        self.set_cookie(cookie);
    }

    fn set_len(len: uint) {
        self.set_header("Content-Length", uint::to_str(len, 10u));
    }

    fn reply_head() {
        let mut rep = [];
        rep += str::bytes(#fmt("HTTP/1.1 %u ", self.code));
        rep += str::bytes(self.status);
        rep += str::bytes("\r\n");

        for self.headers.each { |key, values|
            let lines = vec::map(values) { |value|
                str::bytes(key + ": " + value + "\r\n")
            };

            rep += vec::concat(lines);
        }
        rep += str::bytes("\r\n");

        self.reply(rep);
    }

    fn reply_http<T: to_bytes>(code: uint, status: str, body: T) {
        let body = body.to_bytes();
        self.set_status(code, status);
        self.set_len(body.len());
        self.reply_head();
        self.reply(body);
        self.end();
    }

    fn redirect(location: str) {
        self.set_header("Location", location);
        self.reply_http(302u, "Found", "")
    }

    fn http_100<T: to_bytes>(body: T) {
        self.reply_http(100u, "Continue", body)
    }

    fn http_101<T: to_bytes>(body: T) {
        self.reply_http(101u, "Switching Protocols", body)
    }

    fn http_200<T: to_bytes>(body: T) {
        self.reply_http(200u, "OK", body)
    }

    fn http_201<T: to_bytes>(body: T) {
        self.reply_http(201u, "Created", body)
    }

    fn http_202<T: to_bytes>(body: T) {
        self.reply_http(202u, "Accepted", body)
    }

    fn http_203<T: to_bytes>(body: T) {
        self.reply_http(203u, "Non-Authoritative Information", body)
    }

    fn http_204<T: to_bytes>(body: T) {
        self.reply_http(204u, "No Content", body)
    }

    fn http_205<T: to_bytes>(body: T) {
        self.reply_http(205u, "Reset Content", body)
    }

    fn http_206<T: to_bytes>(body: T) {
        self.reply_http(206u, "Partial Content", body)
    }

    fn http_300<T: to_bytes>(body: T) {
        self.reply_http(300u, "Multiple Choices", body)
    }

    fn http_301<T: to_bytes>(body: T) {
        self.reply_http(301u, "Moved Permanently", body)
    }

    fn http_302<T: to_bytes>(body: T) {
        self.reply_http(302u, "Found", body)
    }

    fn http_303<T: to_bytes>(body: T) {
        self.reply_http(303u, "See Other", body)
    }

    fn http_304<T: to_bytes>(body: T) {
        self.reply_http(304u, "Not Modified", body)
    }

    fn http_305<T: to_bytes>(body: T) {
        self.reply_http(305u, "Use Proxy", body)
    }

    fn http_307<T: to_bytes>(body: T) {
        self.reply_http(305u, "Temporary Redirect", body)
    }

    fn http_400<T: to_bytes>(body: T) {
        self.reply_http(400u, "Bad Request", body)
    }

    fn http_401<T: to_bytes>(body: T) {
        self.reply_http(401u, "Unauthorized", body)
    }

    fn http_402<T: to_bytes>(body: T) {
        self.reply_http(402u, "Payment Required", body)
    }

    fn http_403<T: to_bytes>(body: T) {
        self.reply_http(403u, "Forbidden", body)
    }

    fn http_404<T: to_bytes>(body: T) {
        self.reply_http(404u, "Not Found", body)
    }

    fn http_405<T: to_bytes>(body: T) {
        self.reply_http(405u, "Method Not Allowed", body)
    }

    fn http_406<T: to_bytes>(body: T) {
        self.reply_http(406u, "Not Acceptable", body)
    }

    fn http_407<T: to_bytes>(body: T) {
        self.reply_http(407u, "Proxy Authentication Required", body)
    }

    fn http_408<T: to_bytes>(body: T) {
        self.reply_http(408u, "Request Timeout", body)
    }

    fn http_409<T: to_bytes>(body: T) {
        self.reply_http(409u, "Conflict", body)
    }

    fn http_410<T: to_bytes>(body: T) {
        self.reply_http(410u, "Gone", body)
    }

    fn http_411<T: to_bytes>(body: T) {
        self.reply_http(411u, "Length Required", body)
    }

    fn http_412<T: to_bytes>(body: T) {
        self.reply_http(412u, "Precondition Failed", body)
    }

    fn http_413<T: to_bytes>(body: T) {
        self.reply_http(413u, "Request Entity Too Large", body)
    }

    fn http_414<T: to_bytes>(body: T) {
        self.reply_http(414u, "Request-URI Too Long", body)
    }

    fn http_415<T: to_bytes>(body: T) {
        self.reply_http(415u, "Unsupported Media Type", body)
    }

    fn http_416<T: to_bytes>(body: T) {
        self.reply_http(416u, "Requested Range Not Satisifiable", body)
    }

    fn http_417<T: to_bytes>(body: T) {
        self.reply_http(417u, "Expectation Failed", body)
    }

    fn http_500<T: to_bytes>(body: T) {
        self.reply_http(500u, "Internal Server Error", body)
    }

    fn http_501<T: to_bytes>(body: T) {
        self.reply_http(501u, "Not Implemented", body)
    }

    fn http_502<T: to_bytes>(body: T) {
        self.reply_http(502u, "Bad Gateway", body)
    }

    fn http_503<T: to_bytes>(body: T) {
        self.reply_http(503u, "Service Unavailable", body)
    }

    fn http_504<T: to_bytes>(body: T) {
        self.reply_http(504u, "Gateway Timeout", body)
    }

    fn http_505<T: to_bytes>(body: T) {
        self.reply_http(505u, "HTTP Version Not Supported", body)
    }
}
