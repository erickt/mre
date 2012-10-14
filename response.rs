use cookie::Cookie;

pub type header_map = LinearMap<~str, ~[~str]>;

pub struct Response {
    m2: @mongrel2::Connection,
    req: @mongrel2::Request,
    code: uint,
    status: ~str,
    headers: header_map,
    mut reply: fn@(~[u8]) -> Result<(), ~str>,
    priv mut end_hooks: ~[fn@() -> Result<bool, ~str>],
}

pub fn Response(m2: @mongrel2::Connection, req: @mongrel2::Request) -> Response {
    Response {
        m2: m2,
        req: req,
        code: 200u,
        status: ~"OK",
        headers: LinearMap(),
        reply: |msg: ~[u8]| { m2.reply(req, msg) },
        end_hooks: ~[],
    }
}

fn code_to_status(code: uint) -> ~str {
    // Inspired by Node's http library:
    match code {
        100u => ~"Continue",
        101u => ~"Switching Protocols",
        102u => ~"Processing",              // RFC 2518, obsoleted by RFC 4918
        200u => ~"OK",
        201u => ~"Created",
        202u => ~"Accepted",
        203u => ~"Non-Authoritative Information",
        204u => ~"No Content",
        205u => ~"Reset Content",
        206u => ~"Partial Content",
        207u => ~"Multi-Status",            // RFC 4918
        300u => ~"Multiple Choices",
        301u => ~"Moved Permanently",
        302u => ~"Moved Temporarily",
        303u => ~"See Other",
        304u => ~"Not Modified",
        305u => ~"Use Proxy",
        307u => ~"Temporary Redirect",
        400u => ~"Bad Request",
        401u => ~"Unauthorized",
        402u => ~"Payment Required",
        403u => ~"Forbidden",
        404u => ~"Not Found",
        405u => ~"Method Not Allowed",
        406u => ~"Not Acceptable",
        407u => ~"Proxy Authentication Required",
        408u => ~"Request Time-out",
        409u => ~"Conflict",
        410u => ~"Gone",
        411u => ~"Length Required",
        412u => ~"Precondition Failed",
        413u => ~"Request Entity Too Large",
        414u => ~"Request-URI Too Large",
        415u => ~"Unsupported Media Type",
        416u => ~"Requested Range Not Satisfiable",
        417u => ~"Expectation Failed",
        418u => ~"I'm a teapot",            // RFC 2324
        422u => ~"Unprocessable Entity",    // RFC 4918
        423u => ~"Locked",                  // RFC 4918
        424u => ~"Failed Dependency",       // RFC 4918
        425u => ~"Unordered Collection",    // RFC 4918
        426u => ~"Upgrade Required",        // RFC 2817
        500u => ~"Internal Server Error",
        501u => ~"Not Implemented",
        502u => ~"Bad Gateway",
        503u => ~"Service Unavailable",
        504u => ~"Gateway Time-out",
        505u => ~"HTTP Version not supported",
        506u => ~"Variant Also Negotiates", // RFC 2295
        507u => ~"Insufficient Storage",    // RFC 4918
        509u => ~"Bandwidth Limit Exceeded",
        510u => ~"Not Extended",            // RFC 2774
        _ => ~"unknown",
    }
}

impl Response {
    fn set_status(&mut self, code: uint, status: ~str) {
        self.code = code;
        self.status = status;
    }

    pure fn find_headers(&self, key: &~str) -> Option<&self/~[~str]> {
        self.headers.find_ref(key)
    }

    pure fn find_header(&self, key: &~str) -> Option<~str> {
        match self.find_headers(key) {
            None => None,
            Some(values) => {
                if values.is_empty() {
                    None
                } else {
                    Some(copy values[0u])
                }
            }
        }
    }

    fn set_header(&mut self, name: ~str, value: ~str) {
        let mut values = match self.headers.pop(&name) {
            Some(move values) => values,
            None => ~[],
        };

        values.push(move value);
        self.headers.insert(name, values);
    }

    fn set_cookie(&mut self, cookie: cookie::Cookie) {
        match cookie.to_header() {
            Ok(move header) => self.set_header(~"Set-Cookie", header),
            Err(move e) => fail e,
        }
    }

    fn clear_cookie(&mut self, name: ~str) {
        let mut cookie = cookie::Cookie(name, ~"");
        cookie.max_age = Some(0u);
        self.set_cookie(cookie);
    }

    fn set_len(&mut self, len: uint) {
        self.set_header(~"Content-Length", uint::to_str(len, 10u));
    }

    fn set_content_type(&mut self, content_type: ~str) {
        self.set_header(~"Content-Type", content_type)
    }

    fn each_header(&self, f: fn(&~str, &~[~str]) -> bool) {
        self.headers.each(f)
    }

    fn reply_head(&mut self) {
        let mut rep = ~[];

        // FIXME: https://github.com/mozilla/rust/issues/3722
        let s = fmt!("HTTP/1.1 %u %s\r\n", self.code, self.status);
        unsafe { rep.push_all(str::as_bytes_slice(s)); }

        for self.headers.each |key, values| {
            for values.each |value| {
                // FIXME: https://github.com/mozilla/rust/issues/3722
                let s = fmt!("%s: %s\r\n", *key, *value);
                unsafe { rep.push_all(str::as_bytes_slice(s)); }
            }
        }
        rep.push_all(str::to_bytes("\r\n"));

        self.reply(rep);
    }

    fn reply_http<T: ToBytes>(&mut self, code: uint, body: T) {
        let body = body.to_bytes(false);
        self.set_status(code, code_to_status(code));
        self.set_len(body.len());
        self.reply_head();
        self.reply(body);
        self.end();
    }

    fn reply_text<T: ToBytes>(&mut self, code: uint, body: T) {
        self.set_content_type(~"text/plain");
        self.reply_http(code, body)
    }

    fn reply_html<T: ToBytes>(&mut self, code: uint, body: T) {
        self.set_content_type(~"text/html");
        self.reply_http(code, body)
    }

    fn reply_json<T: ToJson>(&mut self, code: uint, body: T) {
        self.set_content_type(~"application/json");
        self.reply_http(code, body.to_json().to_str())
    }

    fn reply_redirect(&mut self, location: ~str) {
        self.set_header(~"Location", location);
        self.reply_http(302u, "")
    }

    fn add_end_hook(f: fn@() -> Result<bool, ~str>) {
        self.end_hooks.push(f);
    }

    fn end(&mut self) -> Result<(), ~str> {
        self.m2.reply(self.req, &[])
    }
}
