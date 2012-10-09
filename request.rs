use std::sort;

pub enum method {
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

pub type accept = {
    mime_type: (@str, @str),
    quality: float,
    options: @~[@str],
};

pub type request<T> = {
    req: @mongrel2::Request,
    method: method,
    cookies: HashMap<str, cookie::cookie>,
    mut data: T,
    mut _accepts: Option<~[accept]>,
};

pub fn request<T: Copy>(req: @mongrel2::Request, rep: @response, data: T)
  -> Option<@request<T>> {
    let method = match req.headers.find("METHOD") {
        None => {
            rep.reply_http(400u, "");
            return None
        },
        Some(methods) => {
            assert (*methods).len() == 1u;

            match *(*methods)[0u] {
                "HEAD" => HEAD,
                "GET" => GET,
                "POST" => POST,
                "PUT" => PUT,
                "DELETE" => DELETE,
                "TRACE" => TRACE,
                "OPTIONS" => OPTIONS,
                "CONNECT" => CONNECT,
                "PATCH" => PATCH,
                _ => {
                    rep.reply_http(501u, "");
                    return None
                }
            }
        }
    };

    let cookies = match req.headers.find("cookie") {
        None => HashMap(),
        Some(cookies) => {
            match cookie::parse_headers(cookies) {
                Ok(cookies) => cookies,
                Err(e) => {
                    rep.reply_http(400u, e);
                    return None
                }
            }
        }
    };

    Some(@{
        req: req,
        method: method,
        cookies: cookies,
        mut data: data,
        mut _accepts: None,
    })
}

#[doc = "Split a mime string into the mime type and subtype."]
fn parse_mime_type(mime_type: str) -> (@str, @str) {
    let parts = str::splitn_char(mime_type, '/', 1u);
    let typ = copy parts[0];
    let subtyp = if parts.len() == 2u { copy parts[1] } else { "*" };

    (@typ, @subtyp)
}

#[doc = ""]
fn parse_accept_header(header: str) -> ~[accept] {
    let accepts = do header.split_char(',').map |e| {
        let parts = str::replace(e, " ", "").split_char(';');

        let mime_type = parse_mime_type(parts[0]);
        let mut quality = 1.0;
        let mut options = DVec();

        if parts.len() > 1u {
            for parts.view(1, parts.len()).each |option| {
                if option.starts_with("q=") {
                    let q = option.substr(2u, option.len() - 2u);
                    match float::from_str(q) {
                      None => { /* Ignore invalid quality values. */ },
                      Some(q) => quality = q,
                    }
                } else {
                    options.push(@copy option);
                }
            }
        }

        {
            mime_type: mime_type,
            quality: quality,
            options: @vec::from_mut(dvec::unwrap(options))
        }
    };

    // Sort by quality, with highest quality first.
    sort::merge_sort(|e1, e2| e1.quality >= e2.quality, accepts)
}

impl<T> @request<T> {
    fn body() -> @~[u8] {
        self.req.body
    }

    fn path() -> @str {
        self.req.path
    }

    fn is_disconnect() -> bool {
        self.req.is_disconnect()
    }

    fn find_headers(key: str) -> Option<@DVec<@str>> {
        self.req.headers.find(key)
    }

    fn find_header(key: str) -> Option<@str> {
        match self.find_headers(key) {
          None => None,
          Some(values) => { 
            if (*values).len() == 0u {
                None
            } else {
                Some((*values)[0u])
            }
          }
        }
    }

    fn content_type() -> Option<@str> {
        self.find_header("content-type")
    }

    fn accepts() -> ~[accept] {
        // Lazily parse the accept header.
        match self._accepts {
            None => {
                let accepts = match self.find_header("accept") {
                    None => {
                        // If we don't have the header, assume we accept
                        // everything.
                        ~[{
                            mime_type: (@"*", @"*"),
                            quality: 1.0,
                            options: @~[]
                        }]
                    }
                    Some(accept) => parse_accept_header(*accept),
                };

                self._accepts = Some(copy accepts);

                accepts
            }
            Some(accepts) => copy accepts,
        }
    }

    fn accept(mime_type: str) -> bool {
        let (typ, subtyp) = parse_mime_type(mime_type);

        for self.accepts().each |accept| {
            let (t, s) = accept.mime_type;

            if
              (*typ == "*" || typ == t) &&
              (*subtyp == "*" || subtyp == s) {
                return true;
            }
        }

        false
    }

    fn negotiate_media_types<U: Copy>(mime_types: ~[(str, U)]) -> Option<U> {
        let mime_types = do mime_types.map |t_v| {
            match t_v {
              (t, v) => {
                match parse_mime_type(t) {
                  (t, s) => (t, s, v),
                }
              }
            }
        };

        // Walk over the accepts in order and return the first item that
        // matches.
        for self.accepts().each |accept| {
            match accept.mime_type {
              (typ, subtyp) => {
                for mime_types.each |t_s_v| {
                    match t_s_v {
                      (t, s, v) => {
                        if
                         (*typ == "*" || typ == t) &&
                         (*subtyp == "*" || subtyp == s) {
                          return Some(v)
                        }
                      }
                    }
                }
              }
            }
        }

        None
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

        assert parse_accept_header(firefox) == ~[
            {
                mime_type: (@"text", @"html"),
                quality: 1.0,
                options: @~[]
            },
            {
                mime_type: (@"application", @"xhtml+xml"),
                quality: 1.0,
                options: @~[]
            },
            {
                mime_type: (@"application", @"xml"),
                quality: 0.9,
                options: @~[]
            },
            {
                mime_type: (@"*", @"*"),
                quality: 0.8,
                options: @~[]
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

        assert parse_accept_header(webkit) == ~[
            {
                mime_type: (@"application", @"xml"),
                quality: 1.0,
                options: @~[]
            },
            {
                mime_type: (@"application", @"xhtml+xml"),
                quality: 1.0,
                options: @~[]
            },
            {
                mime_type: (@"image", @"png"),
                quality: 1.0,
                options: @~[]
            },
            {
                mime_type: (@"text", @"html"),
                quality: 0.9,
                options: @~[]
            },
            {
                mime_type: (@"text", @"plain"),
                quality: 0.8,
                options: @~[]
            },
            {
                mime_type: (@"*", @"*"),
                quality: 0.5,
                options: @~[]
            }
        ];
    }
}
