use response::Response;

pub enum Method {
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

impl Method: cmp::Eq {
    pure fn eq(other: &Method) -> bool {
        match (self, *other) {
            (HEAD, HEAD)
            | (GET, GET)
            | (POST, POST)
            | (PUT, PUT)
            | (DELETE, DELETE)
            | (TRACE, TRACE)
            | (OPTIONS, OPTIONS)
            | (CONNECT, CONNECT)
            | (PATCH, PATCH) => true,
            _ => false
        }
    }

    pure fn ne(other: &Method) -> bool { !self.eq(other) }
}

pub struct Accept {
    mime_type: (~str, ~str),
    quality: float,
    options: ~[~str],
}

pub struct Request<T> {
    req: @mongrel2::Request,
    method: Method,
    cookies: LinearMap<~str, cookie::Cookie>,
    data: T,
    priv mut _accepts: Option<@~[@Accept]>,
}

pub fn Request<T: Copy>(
    req: @mongrel2::Request,
    rep: @mut Response,
    data: T
) -> Option<Request<T>> {
    let method = match req.headers.find_ref(&~"METHOD") {
        None => {
            rep.reply_http(400, "");
            return None
        },
        Some(methods) => {
            assert methods.len() == 1;

            match methods[0] {
                ~"HEAD" => HEAD,
                ~"GET" => GET,
                ~"POST" => POST,
                ~"PUT" => PUT,
                ~"DELETE" => DELETE,
                ~"TRACE" => TRACE,
                ~"OPTIONS" => OPTIONS,
                ~"CONNECT" => CONNECT,
                ~"PATCH" => PATCH,
                _ => {
                    rep.reply_http(501, "");
                    return None
                }
            }
        }
    };

    let cookies = match req.headers.find_ref(&~"cookie") {
        None => LinearMap(),
        Some(cookies) => {
            match cookie::parse_headers(*cookies) {
                Ok(move cookies) => cookies,
                Err(move e) => {
                    rep.reply_http(400u, e);
                    return None
                }
            }
        }
    };

    Some(Request {
        req: req,
        method: method,
        cookies: cookies,
        data: data,
        _accepts: None,
    })
}

/// Split a mime string into the mime type and subtype.
fn parse_mime_type(mime_type: &str) -> (~str, ~str) {
    let parts = str::splitn_char(mime_type, '/', 1u);
    let typ = copy parts[0];
    let subtyp = if parts.len() == 2u { copy parts[1] } else { ~"*" };

    (typ, subtyp)
}

fn parse_accept_header(header: &str) -> @~[@Accept] {
    let accepts = do header.split_char(',').map |e| {
        let parts = str::replace(*e, ~" ", ~"").split_char(';');

        let mime_type = parse_mime_type(parts[0]);
        let mut quality = 1.0;
        let mut options = ~[];

        if parts.len() > 1u {
            for parts.view(1, parts.len()).each |option| {
                if option.starts_with("q=") {
                    let q = option.substr(2u, option.len() - 2u);
                    match float::from_str(q) {
                      None => { /* Ignore invalid quality values. */ },
                      Some(q) => quality = q,
                    }
                } else {
                    options.push(copy *option);
                }
            }
        }

        @Accept {
            mime_type: mime_type,
            quality: quality,
            options: move options,
        }
    };

    // Sort by quality, with highest quality first.
    @sort::merge_sort(|e1, e2| e1.quality >= e2.quality, accepts)
}

impl Accept: cmp::Eq {
    pure fn eq(other: &Accept) -> bool {
        self.mime_type == other.mime_type &&
        self.quality == other.quality &&
        self.options == other.options
    }

    pure fn ne(other: &Accept) -> bool { !self.eq(other) }
}

impl<T> Request<T> {
    fn body(&self) -> &self/~[u8] {
        &self.req.body
    }

    fn path(&self) -> &self/~str {
        &self.req.path
    }

    fn is_disconnect() -> bool {
        self.req.is_disconnect()
    }

    fn find_headers(&self, key: &~str) -> Option<~[~str]> {
        match self.req.headers.find_ref(key) {
            Some(headers) => Some(copy *headers),
            None => None
        }
    }

    fn find_header(&self, key: &~str) -> Option<~str> {
        match self.find_headers(key) {
          None => None,
          Some(values) => { 
            if values.len() == 0u {
                None
            } else {
                Some(copy values[0u])
            }
          }
        }
    }

    fn find_header(@self, key: &~str) -> Option<~str> {
        match self.find_headers(key) {
          None => None,
          Some(values) => { 
            if values.len() == 0u {
                None
            } else {
                Some(copy values[0u])
            }
          }
        }
    }


    fn content_type() -> Option<~str> {
        self.find_header(&~"content-type")
    }

    fn accepts(&self) -> @~[@Accept] {
        // Lazily parse the accept header.
        match self._accepts {
            Some(accepts) => accepts,
            None => {
                let accepts = match self.find_header(&~"accept") {
                    None => {
                        // If we don't have the header, assume we accept
                        // everything.
                        @~[@Accept {
                            mime_type: (~"*", ~"*"),
                            quality: 1.0,
                            options: ~[]
                        }]
                    }
                    Some(accept) => parse_accept_header(accept),
                };

                self._accepts = Some(move accepts);

                match self._accepts {
                    Some(accepts) => accepts,
                    None => fail,
                }
            }
        }
    }

    fn accept(mime_type: &str) -> bool {
        let (typ, subtyp) = parse_mime_type(mime_type);

        for self.accepts().each |accept| {
            match accept.mime_type {
                (t, s) => {
                    if
                      (typ == ~"*" || typ == t) &&
                      (subtyp == ~"*" || subtyp == s) {
                        return true;
                    }
                }
            }
        }

        false
    }

    fn negotiate_media_types<U: Copy>(mime_types: ~[(~str, U)]) -> Option<U> {
        let mime_types = do mime_types.map |t_v| {
            match t_v {
              &(t, v) => {
                match parse_mime_type(t) {
                  (move t, move s) => (t, s, v),
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
                      &(t, s, v) => {
                        if
                         (typ == ~"*" || typ == t) &&
                         (subtyp == ~"*" || subtyp == s) {
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
        let firefox = ~"text/html,\
                       application/xhtml+xml,\
                       application/xml;q=0.9,\
                       */*;q=0.8";

        assert parse_accept_header(firefox) == @~[
            @Accept {
                mime_type: (~"text", ~"html"),
                quality: 1.0,
                options: ~[]
            },
            @Accept {
                mime_type: (~"application", ~"xhtml+xml"),
                quality: 1.0,
                options: ~[]
            },
            @Accept {
                mime_type: (~"application", ~"xml"),
                quality: 0.9,
                options: ~[]
            },
            @Accept {
                mime_type: (~"*", ~"*"),
                quality: 0.8,
                options: ~[]
            }
        ];
    }

    #[test]
    fn test_parse_accept_header_webkit() {
        let webkit = ~"application/xml,\
                       application/xhtml+xml,\
                       text/html;q=0.9,\
                       text/plain;q=0.8,\
                       image/png,\
                       */*;q=0.5";

        assert parse_accept_header(webkit) == @~[
            @Accept {
                mime_type: (~"application", ~"xml"),
                quality: 1.0,
                options: ~[]
            },
            @Accept {
                mime_type: (~"application", ~"xhtml+xml"),
                quality: 1.0,
                options: ~[]
            },
            @Accept {
                mime_type: (~"image", ~"png"),
                quality: 1.0,
                options: ~[]
            },
            @Accept {
                mime_type: (~"text", ~"html"),
                quality: 0.9,
                options: ~[]
            },
            @Accept {
                mime_type: (~"text", ~"plain"),
                quality: 0.8,
                options: ~[]
            },
            @Accept {
                mime_type: (~"*", ~"*"),
                quality: 0.5,
                options: ~[]
            }
        ];
    }
}
