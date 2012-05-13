import request::request;
import response::response;

type handler<T> = fn@(@request<T>, @response, pcre::match);

iface router<T> {
    fn add(method: str, pattern: str, handler: handler<T>);
    fn add_patterns([(str, str, handler<T>)]);
    fn find(method: str, path: str) -> option<(handler<T>, pcre::match)>;
}

fn router<T>() -> router<T> {
    type routerstate = {
        mut routes: [(str, @pcre::pcre, handler<T>)],
    };

    let router: routerstate = {
        mut routes: []
    };

    impl <T> of router<T> for routerstate {
        fn add(method: str, pattern: str, handler: handler<T>) {
            self.routes += [(method, @pcre::mk_pcre(pattern), handler)];
        }

        fn add_patterns(items: [(str, str, handler<T>)]) {
            vec::iter(items) { |item|
                let (method, pattern, handler) = item;
                self.add(method, pattern, handler)
            };
        }

        fn find(method: str, path: str) -> option<(handler<T>, pcre::match)> {
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

    router as router::<T>
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
