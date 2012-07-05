import cookie::cookie;

type header_map = hashmap<str, @dvec<@str>>;

type response = {
    mut code: uint,
    mut status: @str,
    headers: @header_map,
    mut reply: fn@(@~[u8]),
    mut end: fn@(),
};

fn response(m2: mongrel2::connection, req: @mongrel2::request) -> @response {
    @{
        mut code: 200u,
        mut status: @"OK",
        headers: @str_hash(),
        mut reply: |msg: @~[u8]| { m2.reply(req, *msg); },
        mut end: || { m2.reply(req, str::bytes("")); },
    }
}

fn code_to_status(code: uint) -> str {
    // Inspired by Node's http library:
    alt code {
      100u { "Continue" }
      101u { "Switching Protocols" }
      102u { "Processing" }              // RFC 2518, obsoleted by RFC 4918
      200u { "OK" }
      201u { "Created" }
      202u { "Accepted" }
      203u { "Non-Authoritative Information" }
      204u { "No Content" }
      205u { "Reset Content" }
      206u { "Partial Content" }
      207u { "Multi-Status" }            // RFC 4918
      300u { "Multiple Choices" }
      301u { "Moved Permanently" }
      302u { "Moved Temporarily" }
      303u { "See Other" }
      304u { "Not Modified" }
      305u { "Use Proxy" }
      307u { "Temporary Redirect" }
      400u { "Bad Request" }
      401u { "Unauthorized" }
      402u { "Payment Required" }
      403u { "Forbidden" }
      404u { "Not Found" }
      405u { "Method Not Allowed" }
      406u { "Not Acceptable" }
      407u { "Proxy Authentication Required" }
      408u { "Request Time-out" }
      409u { "Conflict" }
      410u { "Gone" }
      411u { "Length Required" }
      412u { "Precondition Failed" }
      413u { "Request Entity Too Large" }
      414u { "Request-URI Too Large" }
      415u { "Unsupported Media Type" }
      416u { "Requested Range Not Satisfiable" }
      417u { "Expectation Failed" }
      418u { "I'm a teapot" }            // RFC 2324
      422u { "Unprocessable Entity" }    // RFC 4918
      423u { "Locked" }                  // RFC 4918
      424u { "Failed Dependency" }       // RFC 4918
      425u { "Unordered Collection" }    // RFC 4918
      426u { "Upgrade Required" }        // RFC 2817
      500u { "Internal Server Error" }
      501u { "Not Implemented" }
      502u { "Bad Gateway" }
      503u { "Service Unavailable" }
      504u { "Gateway Time-out" }
      505u { "HTTP Version not supported" }
      506u { "Variant Also Negotiates" } // RFC 2295
      507u { "Insufficient Storage" }    // RFC 4918
      509u { "Bandwidth Limit Exceeded" }
      510u { "Not Extended" }            // RFC 2774
      _ { "unknown" }
    }
}

impl response for @response {
    fn set_status(code: uint, +status: str) {
        self.code = code;
        self.status = @status;
    }

    fn find_headers(key: str) -> option<@dvec<@str>> {
        (*self.headers).find(key)
    }

    fn find_header(key: str) -> option<@str> {
        do self.find_headers(key).chain |values| {
            if (*values).len() == 0u {
                none
            } else {
                some((*values)[0u])
            }
        }
    }

    fn set_header(+name: str, +value: str) {
        let mut values = alt (*self.headers).find(name) {
          none {
            let values = @dvec();
            (*self.headers).insert(name, values);
            values
          }
          some(values) { values }
        };

        (*values).push(@value);
    }

    fn set_cookie(cookie: cookie::cookie) {
        alt cookie.to_header() {
          ok(header) { self.set_header("Set-Cookie", copy header) }
          err(e) { fail e; }
        }
    }

    fn clear_cookie(name: @str) {
        let cookie = cookie::cookie(name, @"");
        cookie.max_age = some(0u);
        self.set_cookie(cookie);
    }

    fn set_len(len: uint) {
        self.set_header("Content-Length", uint::to_str(len, 10u));
    }

    fn set_content_type(+content_type: str) {
        self.set_header("Content-Type", content_type)
    }

    fn each_header(f: fn(&&str, &&@dvec<@str>) -> bool) {
        (*self.headers).each(f)
    }

    fn reply_head() {
        let mut rep = dvec();
        rep.push_all(str::bytes(#fmt("HTTP/1.1 %u %s\r\n",
                                     self.code,
                                     *self.status)));

        for (*self.headers).each |key, values| {
            for (*values).each |value| {
                rep.push_all(str::bytes(key + ": " + *value + "\r\n"));
            }
        }
        rep.push_all(str::bytes("\r\n"));

        self.reply(@vec::from_mut(dvec::unwrap(rep)));
    }

    fn reply_http<T: to_bytes>(code: uint, body: T) {
        let body = body.to_bytes();
        self.set_status(code, code_to_status(code));
        self.set_len(body.len());
        self.reply_head();
        self.reply(@body);
        self.end();
    }

    fn reply_text<T: to_bytes>(code: uint, body: T) {
        self.set_content_type("text/plain");
        self.reply_http(code, body)
    }

    fn reply_html<T: to_bytes>(code: uint, body: T) {
        self.set_content_type("text/html");
        self.reply_http(code, body)
    }

    fn reply_json<T: to_json>(code: uint, body: T) {
        self.set_content_type("application/json");
        self.reply_http(code, json::to_str(body.to_json()))
    }

    fn reply_redirect(+location: str) {
        self.set_header("Location", location);
        self.reply_http(302u, "")
    }
}
