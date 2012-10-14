pub struct Cookie {
    name: ~str,
    value: ~str,
    path: Option<~str>,
    domain: Option<~str>,
    max_age: Option<uint>,
    secure: bool,
    http_only: bool
}

pub fn Cookie(name: ~str, value: ~str) -> Cookie {
    Cookie {
        name: name,
        value: value,
        path: None,
        domain: None,
        max_age: None,
        secure: false,
        http_only: false
    }
}

impl Cookie {
    fn to_header() -> Result<~str, ~str> {
        // FIXME: Move error checking to the constructor.

        if !cookie_parser::is_name(self.name) {
            return Err(~"invalid name");
        }

        let mut cookie = if self.value.is_empty() {
            self.name + "="
        } else {
            if !cookie_parser::is_value(self.value) {
                return Err(~"invalid value");
            }

            fmt!("%s=%s", self.name, self.value)
        };

        match self.domain {
            None => { },
            Some(domain) => {
                if cookie_parser::is_domain(domain) {
                    cookie += fmt!("; domain=%s", domain);
                } else {
                    return Err(~"invalid domain");
                }
            }
        }

        match self.path {
            None => { }
            Some(path) => {
                if cookie_parser::is_path(path) {
                    cookie += fmt!("; path=%s", path);
                } else {
                    return Err(~"invalid path");
                }
            }
        }

        match copy self.max_age {
            None => { }
            Some(max_age) => {
                let tm = if max_age == 0u {
                    std::time::at({ sec: 0_i64, nsec: 0_i32 })
                } else {
                    let t = std::time::get_time();
                    std::time::at_utc({ sec: t.sec + max_age as i64, nsec: 0_i32 })
                };

                // Not every browser supports max-age...
                //vec::push(cookie, "max-age=" + uint::str(max_age));

                cookie += fmt!("; expires=%s", tm.rfc822());
            }
        }

        if self.secure    { cookie += "; Secure"; }
        if self.http_only { cookie += "; HttpOnly"; }

        Ok(cookie)
    }
}

pub fn parse_header(header: &str) -> Result<~[Cookie], ~str> {
    let header = header.trim();

    // Exit early if empty.
    if header.is_empty() { return Err(~"empty cookie") }

    let mut cookies = ~[];

    for header.split_char(';').each |line| {
        let parts = str::splitn_char(*line, '=', 1u);

        let (name, value) = if parts.len() == 1u {
            return Err(~"empty cookie value")
        } else {
            (parts[0u].trim(), parts[1u].trim())
        };

        if !cookie_parser::is_name(name) {
            return Err(fmt!("invalid cookie name: %?", name));
        }

        if !cookie_parser::is_value(value) {
            return Err(fmt!("invalid cookie value: %?", value));
        }

        cookies.push(Cookie(name, value));
    }

    Ok(cookies)
}

pub fn parse_headers(headers: &[~str]) -> Result<LinearMap<~str, Cookie>, ~str> {
    let mut cookies = LinearMap();

    for headers.each |header| {
        match parse_header(*header) {
            Ok(move cs) => {
                do vec::consume(cs) |_i, cookie| {
                    cookies.insert(copy cookie.name, move cookie);
                }
            },
            Err(move e) => return Err(e),
        }
    }

    Ok(cookies)
}

#[doc = "
Helper functions for parsing cookies according to RFC 6265, Section 4.1.
"]
mod cookie_parser {
    pub fn is_name(name: &str) -> bool {
        http_parser::is_token(name)
    }

    pub fn is_cookie_octet(ch: char) -> bool {
        if !char::is_ascii(ch) { return false; }

        match ch {
            '\x21' | '\x23' .. '\x2b' | '\x2D' .. '\x3A' | '\x3C' .. '\x5B'
          | '\x5D' .. '\x7E' => true,
          _ => false,
        }
    }

    pub fn is_value(value: &str) -> bool {
        let mut pos = 0u;
        let len = value.len();

        // Exit early if we have an empty string.
        if len == 0u { return true; }

        // Check if the value is surrounded by double quotes.
        let {ch, next} = str::char_range_at(value, pos);

        let quoted = if ch == '"' {
            pos = next;
            if pos == len { return false; }

            true
        } else {
            false
        };

        while pos < len {
            let {ch, next} = str::char_range_at(value, pos);

            if quoted && ch == '"' {
                return next == len;
            }

            if !is_cookie_octet(ch) { return false; }

            pos = next;
        }

        !quoted && pos == len
    }

    pub fn is_domain(_domain: &str) -> bool {
        // FIXME: Actually implement.
        true
    }

    pub fn is_path(path: &str) -> bool {
        for str::each_char(path) |ch| {
            if !char::is_ascii(ch) || http_parser::is_ctl(ch) || ch == ';' {
                return false;
            }
        }

        true
    }
}



#[doc = "
Helper functions for parsing http according to RFC 2616, Section 2.2.
"]
mod http_parser {
    pub fn is_char(ch: char) -> bool {
        char::is_ascii(ch)
    }

    pub fn is_ctl(ch: char) -> bool {
        match ch {
          '\x00' .. '\x1f' | '\x7f' => true,
          _ => false,
        }
    }

    pub fn is_separator(ch: char) -> bool {
        match ch {
            '(' | ')' | '<' | '>'  | '@'
          | ',' | ';' | ':' | '\\' | '"'
          | '/' | '[' | ']' | '?'  | '='
          | '{' | '}' | ' ' | '\t' => true,
          _ => false,
        }
    }

    pub fn is_token(token: &str) -> bool {
        if token.len() == 0u { return false; }

        for str::each_char(token) |ch| {
            if !is_char(ch) || is_ctl(ch) || is_separator(ch) {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_is_cookie_octet() {
        assert  cookie_parser::is_cookie_octet('A');
        assert !cookie_parser::is_cookie_octet('"');
    }

    #[test]
    fn test_is_value() {
        assert !cookie_parser::is_value("\"");
        assert !cookie_parser::is_value("\"a");
        assert !cookie_parser::is_value("foo bar");
        assert !cookie_parser::is_value("foo\"");
        assert !cookie_parser::is_value("\"foo");

        assert cookie_parser::is_value("");
        assert cookie_parser::is_value("\"\"");
        assert cookie_parser::is_value("foo");;
        assert cookie_parser::is_value("\"foo\"");;
    }
}
