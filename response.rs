import cookie::cookie;

type response = {
    mut code: uint,
    mut status: str,
    headers: hashmap<str, [str]>,
    mut reply: fn@([u8]),
    mut end: fn@(),
};

fn response(m2: mongrel2::connection, req: mongrel2::request) -> @response {
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

    fn reply_http(code: uint, status: str, body: [u8]) {
        self.set_status(code, status);
        self.set_len(body.len());
        self.reply_head();
        self.reply(body);
        self.end();
    }

    fn redirect(location: str) {
        self.set_header("Location", location);
        self.reply_http(302u, "Found", [])
    }

    fn http_100() {
        self.reply_http(100u, "Continue", [])
    }

    fn http_101() {
        self.reply_http(101u, "Switching Protocols", [])
    }

    fn http_200(body: [u8]) {
        self.reply_http(200u, "OK", body)
    }

    fn http_201() {
        self.reply_http(201u, "Created", [])
    }

    fn http_202() {
        self.reply_http(202u, "Accepted", [])
    }

    fn http_203() {
        self.reply_http(203u, "Non-Authoritative Information", [])
    }

    fn http_204() {
        self.reply_http(204u, "No Content", [])
    }

    fn http_205() {
        self.reply_http(205u, "Reset Content", [])
    }

    fn http_206() {
        self.reply_http(206u, "Partial Content", [])
    }

    fn http_300() {
        self.reply_http(300u, "Multiple Choices", [])
    }

    fn http_301() {
        self.reply_http(301u, "Moved Permanently", [])
    }

    fn http_302() {
        self.reply_http(302u, "Found", [])
    }

    fn http_303() {
        self.reply_http(303u, "See Other", [])
    }

    fn http_304() {
        self.reply_http(304u, "Not Modified", [])
    }

    fn http_305() {
        self.reply_http(305u, "Use Proxy", [])
    }

    fn http_307() {
        self.reply_http(305u, "Temporary Redirect", [])
    }

    fn http_400(body: [u8]) {
        self.reply_http(400u, "Bad Request", body)
    }

    fn http_401() {
        self.reply_http(401u, "Unauthorized", [])
    }

    fn http_402() {
        self.reply_http(402u, "Payment Required", [])
    }

    fn http_403() {
        self.reply_http(403u, "Forbidden", [])
    }

    fn http_404() {
        self.reply_http(404u, "Not Found", [])
    }

    fn http_405() {
        self.reply_http(405u, "Method Not Allowed", [])
    }

    fn http_406() {
        self.reply_http(406u, "Not Acceptable", [])
    }

    fn http_407() {
        self.reply_http(407u, "Proxy Authentication Required", [])
    }

    fn http_408() {
        self.reply_http(408u, "Request Timeout", [])
    }

    fn http_409() {
        self.reply_http(409u, "Conflict", [])
    }

    fn http_410() {
        self.reply_http(410u, "Gone", [])
    }

    fn http_411() {
        self.reply_http(411u, "Length Required", [])
    }

    fn http_412() {
        self.reply_http(412u, "Precondition Failed", [])
    }

    fn http_413() {
        self.reply_http(413u, "Request Entity Too Large", [])
    }

    fn http_414() {
        self.reply_http(414u, "Request-URI Too Long", [])
    }

    fn http_415() {
        self.reply_http(415u, "Unsupported Media Type", [])
    }

    fn http_416() {
        self.reply_http(416u, "Requested Range Not Satisifiable", [])
    }

    fn http_417() {
        self.reply_http(417u, "Expectation Failed", [])
    }

    fn http_500(body: [u8]) {
        self.reply_http(500u, "Internal Server Error", body)
    }

    fn http_501() {
        self.reply_http(501u, "Not Implemented", [])
    }

    fn http_502() {
        self.reply_http(502u, "Bad Gateway", [])
    }

    fn http_503() {
        self.reply_http(503u, "Service Unavailable", [])
    }

    fn http_504() {
        self.reply_http(504u, "Gateway Timeout", [])
    }

    fn http_505() {
        self.reply_http(505u, "HTTP Version Not Supported", [])
    }
}
