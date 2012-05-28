import std::sort;

export method;
export accept;
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

type accept = {
    mime_type: (str, str),
    quality: float,
    options: [str],
};

type request<T> = {
    req: @mongrel2::request,
    method: method,
    cookies: hashmap<str, cookie::cookie>,
    mut data: T,
    mut _accepts: option<[accept]>,
};

fn request<T: copy>(req: @mongrel2::request, rep: @response, data: T)
  -> option<@request<T>> {
    let method = alt req.headers.find("METHOD") {
      none {
        rep.reply_http(400u, "");
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
            rep.reply_http(501u, "");
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
            rep.reply_http(400u, e);
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
        mut _accepts: none,
    })
}

#[doc = "Split a mime string into the mime type and subtype."]
fn parse_mime_type(mime_type: str) -> (str, str) {
    let parts = str::splitn_char(mime_type, '/', 1u);
    let typ = parts[0];
    let subtyp = if parts.len() == 2u { parts[1] } else { "*" };

    (typ, subtyp)
}

#[doc = ""]
fn parse_accept_header(header: str) -> [accept] {
    let accepts = header.split_char(',').map { |e|
        let parts = str::replace(e, " ", "").split_char(';');

        let mime_type = parse_mime_type(parts[0]);
        let mut quality = 1.0;
        let mut options = [];

        if parts.len() > 1u {
            vec::iter_between(parts, 1u, parts.len()) { |option|
                if option.starts_with("q=") {
                    let q = option.substr(2u, option.len() - 2u);
                    alt float::from_str(q) {
                      none { /* Ignore invalid quality values. */ }
                      some(q) { quality = q; }
                    }
                } else {
                    vec::push(options, option);
                }
            }
        }

        { mime_type: mime_type, quality: quality, options: options }
    };

    // Sort by quality, with highest quality first.
    sort::merge_sort({ |e1, e2| e1.quality >= e2.quality }, accepts)
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

    fn content_type() -> option<str> {
        self.find_header("content-type")
    }

    fn accepts() -> [accept] {
        // Lazily parse the accept header.
        alt self._accepts {
          none {
            let accepts = alt self.find_header("accept") {
              none {
                // If we don't have the header, assume we accept everything.
                [{ mime_type: ("*", "*"), quality: 1.0, options: [] }]
              }
              some(accept) { parse_accept_header(accept) }
            };

            self._accepts = some(accepts);

            accepts
          }
          some(accepts) { accepts }
        }
    }

    fn accept(mime_type: str) -> bool {
        let mime_type = parse_mime_type(mime_type);

        for self.accepts().each { |accept|
            if accept.mime_type == ("*", "*") { ret true; }
            if accept.mime_type == mime_type { ret true; }
        }

        false
    }

    fn negotiate_media_types<U: copy>(mime_types: [(str, U)]) -> option<U> {
        let mime_types = mime_types.map { |t_v|
            let (t, v) = t_v;
            let (t, s) = parse_mime_type(t);
            (t, s, v)
        };

        // Walk over the accepts in order and return the first item that
        // matches.
        for self.accepts().each { |accept|
            let (typ, subtyp) = accept.mime_type;

            for mime_types.each { |t_s_v|
                let (t, s, v) = t_s_v;

                #error("%? %? -> %? %?", typ, subtyp, t, s);

                if typ == "*" || typ == t {
                    if subtyp == "*" || subtyp == s {
                        ret some(v)
                    }
                }
            }
        }

        none
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_parse_accept_header_firefox() {
        let firefox = "text/html," +
                      "application/xhtml+xml," +
                      "application/xml;q=0.9," +
                      "*/*;q=0.8";

        assert parse_accept_header(firefox) == [
            {
                mime_type: ("text", "html"),
                quality: 1.0,
                options: []
            },
            {
                mime_type: ("application", "xhtml+xml"),
                quality: 1.0,
                options: []
            },
            {
                mime_type: ("application", "xml"),
                quality: 0.9,
                options: []
            },
            {
                mime_type: ("*", "*"),
                quality: 0.8,
                options: []
            }
        ];
    }

    #[test]
    fn test_parse_accept_header_webkit() {
        let webkit = "application/xml," +
                     "application/xhtml+xml," +
                     "text/html;q=0.9," +
                     "text/plain;q=0.8," +
                     "image/png," +
                     "*/*;q=0.5";

        assert parse_accept_header(webkit) == [
            {
                mime_type: ("application", "xml"),
                quality: 1.0,
                options: []
            },
            {
                mime_type: ("application", "xhtml+xml"),
                quality: 1.0,
                options: []
            },
            {
                mime_type: ("image", "png"),
                quality: 1.0,
                options: []
            },
            {
                mime_type: ("text", "html"),
                quality: 0.9,
                options: []
            },
            {
                mime_type: ("text", "plain"),
                quality: 0.8,
                options: []
            },
            {
                mime_type: ("*", "*"),
                quality: 0.5,
                options: []
            }
        ];
    }
}
