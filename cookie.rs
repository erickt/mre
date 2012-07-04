import std::time;
import std::time::tm;

type cookie = @{
    name: @str,
    value: @str,
    mut path: option<@str>,
    mut domain: option<@str>,
    mut max_age: option<uint>,
    mut secure: bool,
    mut http_only: bool
};

fn cookie(name: @str, value: @str) -> cookie {
    @{
        name: name,
        value: value,
        mut path: none,
        mut domain: none,
        mut max_age: none,
        mut secure: false,
        mut http_only: false
    }
}

impl cookie for cookie {
    fn to_header() -> result<str, str> {
        // FIXME: Move error checking to the constructor.

        if !cookie_parser::is_name(*self.name) {
            ret err("invalid name");
        }

        let mut cookie = if *self.value == "" {
            *self.name + "="
        } else {
            if !cookie_parser::is_value(*self.value) {
                ret err("invalid value");
            }

            *self.name + "=" + *self.value
        };

        alt copy self.domain {
          none {}
          some(domain) {
            if cookie_parser::is_domain(domain) {
                cookie += "; domain=" + *domain;
            } else {
                ret err("invalid domain");
            }
          }
        }

        alt copy self.path {
          none {}
          some(path) {
            if cookie_parser::is_path(path) {
                cookie += "; path=" + *path;
            } else {
                ret err("invalid path");
            }
          }
        }

        alt copy self.max_age {
          none {}
          some(max_age) {
            let tm = if max_age == 0u {
                std::time::at({ sec: 0_i64, nsec: 0_i32 })
            } else {
                let t = std::time::get_time();
                std::time::at_utc({ sec: t.sec + max_age as i64, nsec: 0_i32 })
            };

            // Not every browser supports max-age...
            //vec::push(cookie, "max-age=" + uint::str(max_age));

            cookie += "; expires=" + tm.rfc822();
          }
        }

        if self.secure    { cookie += "; Secure"; }
        if self.http_only { cookie += "; HttpOnly"; }

        ok(cookie)
    }
}

fn parse_header(header: str) -> result<@~[cookie], str> {
    let header = header.trim();

    // Exit early if empty.
    if header == "" { ret err("empty cookie") }

    let cookies = dvec();

    for header.split_char(';').each |line| {
        let parts = str::splitn_char(line, '=', 1u);

        let (name, value) = if parts.len() == 1u {
            ret err("empty cookie value")
        } else {
            (parts[0u].trim(), parts[1u].trim())
        };

        if !cookie_parser::is_name(name) {
            ret err(#fmt("invalid cookie name: %?", name));
        }

        if !cookie_parser::is_value(value) {
            ret err(#fmt("invalid cookie value: %?", value));
        }

        cookies.push(cookie(@name, @value));
    }

    ok(@vec::from_mut(dvec::unwrap(cookies)))
}

fn parse_headers(headers: @dvec<@str>) -> result<hashmap<str, cookie>, str> {
    let mut cookies = str_hash();

    for (*headers).each |header| {
        alt parse_header(*header) {
          ok(cs) {
            for (*cs).each |cookie| {
                cookies.insert(copy *cookie.name, cookie);
            }
          }
          err(e) { ret err(copy e); }
        }
    }

    ok(cookies)
}

#[doc = "
Helper functions for parsing cookies according to RFC 6265, Section 4.1.
"]
mod cookie_parser {
    fn is_name(name: str) -> bool {
        http_parser::is_token(name)
    }

    fn is_cookie_octet(ch: char) -> bool {
        if !char::is_ascii(ch) { ret false; }

        alt ch {
            '\x21' | '\x23' to '\x2b' | '\x2D' to '\x3A' | '\x3C' to '\x5B'
          | '\x5D' to '\x7E' { true }
          _ { false }
        }
    }

    fn is_value(value: str) -> bool {
        let mut pos = 0u;
        let len = value.len();

        // Exit early if we have an empty string.
        if len == 0u { ret true; }

        // Check if the value is surrounded by double quotes.
        let {ch, next} = str::char_range_at(value, pos);

        let quoted = if ch == '"' {
            pos = next;
            if pos == len { ret false; }

            true
        } else {
            false
        };

        while pos < len {
            let {ch, next} = str::char_range_at(value, pos);

            if quoted && ch == '"' {
                ret next == len;
            }

            if !is_cookie_octet(ch) { ret false; }

            pos = next;
        }

        !quoted && pos == len
    }

    fn is_domain(_domain: @str) -> bool {
        // FIXME: Actually implement.
        true
    }

    fn is_path(path: @str) -> bool {
        for (*path).each_char |ch| {
            if !char::is_ascii(ch) || http_parser::is_ctl(ch) || ch == ';' {
                ret false;
            }
        }

        true
    }
}



#[doc = "
Helper functions for parsing http according to RFC 2616, Section 2.2.
"]
mod http_parser {
    fn is_char(ch: char) -> bool {
        char::is_ascii(ch)
    }

    fn is_ctl(ch: char) -> bool {
        alt ch {
          '\x00' to '\x1f' | '\x7f' { true }
          _ { false }
        }
    }

    fn is_separator(ch: char) -> bool {
        alt ch {
            '(' | ')' | '<' | '>'  | '@'
          | ',' | ';' | ':' | '\\' | '"'
          | '/' | '[' | ']' | '?'  | '='
          | '{' | '}' | ' ' | '\t' { true }
          _ { false }
        }
    }

    fn is_token(token: str) -> bool {
        if token.len() == 0u { ret false; }

        for token.each_char |ch| {
            if !is_char(ch) || is_ctl(ch) || is_separator(ch) {
                ret false;
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
