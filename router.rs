import request::request;
import response::response;

type handler = fn@(@request, @response, pcre::match);

type router = {
    mut routes: [(str, @pcre::pcre, handler)],
};

fn router() -> router {
    { mut routes: [] }
}

impl router for router {
    fn add(method: str, pattern: str, handler: handler) {
        self.routes += [(method, @pcre::mk_pcre(pattern), handler)];
    }

    fn add_patterns(items: [(str, str, handler)]) {
        vec::iter(items) { |item|
            let (method, pattern, handler) = item;
            self.add(method, pattern, handler)
        };
    }

    fn find(method: str, path: str) -> option<(handler, pcre::match)> {
        for self.routes.each() { |item|
            let (meth, regex, handler) = item;

            if method == meth {
                let m = (*regex).match(path);
                if m.matched() {
                    ret some((handler, m));
                }
            }
        }
        none
    }
}

#[cfg(test)]
mod tests {
    import pcre::match;

    fn check_path(router: router, method: str, path: str, f: handler,
                  captures: [str]) {
        let (handler, m) = router.find(method, path).get();
        assert handler == f;
        assert m.substrings() == captures;
    }

    #[test]
    fn test_router() {
        let router = router();
        router.find("GET", "") == none;
        router.find("GET", "/foo/bar/baz") == none;

        let a = { |req, _m| response::http_200(req, []) };
        let b = { |req, _m| response::http_200(req, []) };
        let c = { |req, _m| response::http_200(req, []) };
        let d = { |req, _m| response::http_200(req, []) };
        let z = { |req, _m| response::http_200(req, []) };

        router.add_patterns([
            ("GET", "^/$", a),
            ("GET", "^/foo$", b),
            ("GET", "^/foo/bar/baz$", c),
            ("GET", "^/([^\\/]+)/(.*)$", d),
            ("GET", "", z)
        ]);

        check_path(router, "GET", "/", a, []);
        check_path(router, "GET", "/foo", b, []);
        check_path(router, "GET", "/foo/bar/baz", c, []);
        check_path(router, "GET", "/a12/b34", d, ["a12", "b34"]);
        check_path(router, "GET", "/a12/b34/c/d", d, ["a12", "b34/c/d"]);
        check_path(router, "GET", "lalala", z, []);


    }
}
