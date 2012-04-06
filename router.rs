import request::request;
import response::response;

type handler = fn@(request, pcre::match) -> response;

iface router {
    fn add(method: str, pattern: str, handler: handler);
    fn add_patterns([(str, str, handler)]);
    fn find(method: str, path: str) -> option<(handler, pcre::match)>;
}

fn router() -> router {
    type routerstate = {
        mut routes: [(str, @pcre::pcre, handler)],
    };

    let router: routerstate = {
        mut routes: []
    };

    impl of router for routerstate {
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
            for item in self.routes {
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

    router as router
}

#[cfg(test)]
mod tests {
    import pcre::match;

    fn check_path(router: router, path: str, f: handler, captures: [str]) {
        let (handler, m) = option::get(router.find(path));
        assert handler == f;
        assert m.substrings() == captures;
    }

    #[test]
    fn test_router() {
        let router = router();
        router.find("") == none;
        router.find("/foo/bar/baz") == none;

        let a = { |req, _m| response::http_200(req, []) };
        let b = { |req, _m| response::http_200(req, []) };
        let c = { |req, _m| response::http_200(req, []) };
        let d = { |req, _m| response::http_200(req, []) };
        let z = { |req, _m| response::http_200(req, []) };

        router.add_patterns([
            ("^/$", a),
            ("^/foo$", b),
            ("^/foo/bar/baz$", c),
            ("^/([^\\/]+)/(.*)$", d),
            ("", z)
        ]);

        check_path(router, "/", a, []);
        check_path(router, "/foo", b, []);
        check_path(router, "/foo/bar/baz", c, []);
        check_path(router, "/a12/b34", d, ["a12", "b34"]);
        check_path(router, "/a12/b34/c/d", d, ["a12", "b34/c/d"]);
        check_path(router, "lalala", z, []);


    }
}
