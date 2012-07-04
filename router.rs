import request::{method, request};
import response::response;

type handler<T> = fn@(@request<T>, @response, pcre::match);

type router<T> = {
    routes: dvec<(method, @pcre::pcre, handler<T>)>,
};

fn router<T>() -> router<T> {
    { routes: dvec() }
}

impl router<T> for router<T> {
    fn add(method: method, pattern: str, handler: handler<T>) {
        let regex = pcre::pcre(pattern);

        if regex.is_err() {
            fail *regex.get_err();
        }
        
        self.routes.push((method, @result::unwrap(regex), handler));
    }

    fn add_patterns(items: ~[(method, str, handler<T>)]) {
        for items.each |item| {
            alt item {
              (method, pattern, handler) {
                self.add(method, pattern, handler)
              }
            }
        };
    }

    fn find(method: method, path: str) -> option<(handler<T>, pcre::match)> {
        for self.routes.each() |item| {
            let (meth, regex, handler) = item;

            if method == meth {
                alt (*regex).match(path) {
                  none { }
                  some(m) { ret some((handler, m)); }
                }
            }
        }
        none
    }
}

#[cfg(test)]
mod tests {
    import pcre::match;
    import request::GET;

    fn check_path<T>(router: router<T>, method: method, path: str,
                     f: handler<T>, captures: @~[@str]) {
        let (handler, m) = router.find(method, path).get();
        assert handler == f;
        assert m.substrings == captures;
    }

    #[test]
    fn test_router() {
        let router = router::<()>();
        router.find(GET, "") == none;
        router.find(GET, "/foo/bar/baz") == none;

        let a = { |_req, rep: @response, _m| rep.reply_http(200u, "") };
        let b = { |_req, rep: @response, _m| rep.reply_http(200u, "") };
        let c = { |_req, rep: @response, _m| rep.reply_http(200u, "") };
        let d = { |_req, rep: @response, _m| rep.reply_http(200u, "") };
        let z = { |_req, rep: @response, _m| rep.reply_http(200u, "") };

        router.add_patterns(~[
            (GET, "^/$", a),
            (GET, "^/foo$", b),
            (GET, "^/foo/bar/baz$", c),
            (GET, "^/([^\\/]+)/(.*)$", d),
            (GET, "", z)
        ]);

        check_path(router, GET, "/", a, @~[]);
        check_path(router, GET, "/foo", b, @~[]);
        check_path(router, GET, "/foo/bar/baz", c, @~[]);
        check_path(router, GET, "/a12/b34", d, @~[@"a12", @"b34"]);
        check_path(router, GET, "/a12/b34/c/d", d, @~[@"a12", @"b34/c/d"]);
        check_path(router, GET, "lalala", z, @~[]);
    }
}
