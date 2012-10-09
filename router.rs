use request::{method, request};
use response::response;

pub type handler<T> = fn@(@request<T>, @response, pcre::Match);

pub type router<T> = {
    routes: DVec<(method, @pcre::Pcre, handler<T>)>,
};

pub fn router<T>() -> router<T> {
    { routes: DVec() }
}

impl<T> router<T> {
    fn add(method: method, pattern: str, handler: handler<T>) {
        let regex = pcre::Pcre(pattern);

        if regex.is_err() {
            fail *regex.get_err();
        }
        
        self.routes.push((method, @result::unwrap(regex), handler));
    }

    fn add_patterns(items: ~[(method, str, handler<T>)]) {
        for items.each |item| {
            match item {
              (method, pattern, handler) => {
                self.add(method, pattern, handler)
              }
            }
        };
    }

    fn find(method: method, path: str) -> Option<(handler<T>, pcre::Match)> {
        for self.routes.each() |item| {
            let (meth, regex, handler) = item;

            if method == meth {
                match regex.exec(path) {
                  None => { },
                  Some(m) => return Some((handler, m)),
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use pcre::Match;
    use request::GET;

    fn check_path<T>(router: router<T>, method: method, path: str,
                     f: handler<T>, captures: @~[@str]) {
        let (handler, m) = router.find(method, path).get();
        assert handler == f;
        assert m.substrings == captures;
    }

    #[test]
    fn test_router() {
        let router = router::<()>();
        router.find(GET, "") == None;
        router.find(GET, "/foo/bar/baz") == None;

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
