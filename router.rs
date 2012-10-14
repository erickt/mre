use request::{Method, Request};
use response::Response;

pub type Handler<T> = fn@(@Request<T>, @mut Response, pcre::Match);

pub struct Router<T> {
    routes: DVec<(Method, pcre::Pcre, Handler<T>)>,
}

pub fn Router<T>() -> Router<T> {
    Router { routes: DVec() }
}

impl<T> Router<T> {
    fn add(method: Method, pattern: &str, handler: Handler<T>) {
        let regex = match pcre::Pcre(pattern) {
            Ok(move regex) => regex,
            Err(move e) => fail e,
        };
        
        self.routes.push((method, regex, handler));
    }

    fn add_patterns(items: ~[(Method, ~str, Handler<T>)]) {
        for items.each |item| {
            match item {
              &(method, pattern, handler) => {
                self.add(method, pattern, handler)
              }
            }
        };
    }

    fn find(method: Method, path: &str) -> Option<(Handler<T>, pcre::Match)> {
        for self.routes.each() |item| {
            match item {
                &(meth, regex, handler) => {
                    if method == meth {
                        match regex.exec(path) {
                          None => { },
                          Some(move m) => return Some((handler, m)),
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
    use pcre::Match;
    use request::GET;

    fn check_path<T>(router: &Router<T>, method: Method, path: ~str,
                     captures: ~[@~str]) {
        let (_handler, m) = option::unwrap(router.find(method, path));
        assert m.substrings == captures;
    }

    #[test]
    fn test_router() {
        let router = @Router::<()>();
        assert router.find(GET, "").is_none();
        assert router.find(GET, "/foo/bar/baz").is_none();

        let a = |_req, rep: @mut Response, _m| { rep.reply_http(200u, "") };
        let b = |_req, rep: @mut Response, _m| { rep.reply_http(200u, "") };
        let c = |_req, rep: @mut Response, _m| { rep.reply_http(200u, "") };
        let d = |_req, rep: @mut Response, _m| { rep.reply_http(200u, "") };
        let z = |_req, rep: @mut Response, _m| { rep.reply_http(200u, "") };

        router.add_patterns(~[
            (GET, ~"^/$", a),
            (GET, ~"^/foo$", b),
            (GET, ~"^/foo/bar/baz$", c),
            (GET, ~"^/([^\\/]+)/(.*)$", d),
            (GET, ~"", z)
        ]);

        check_path(router, GET, ~"/", ~[]);
        check_path(router, GET, ~"/foo", ~[]);
        check_path(router, GET, ~"/foo/bar/baz", ~[]);
        check_path(router, GET, ~"/a12/b34", ~[@~"a12", @~"b34"]);
        check_path(router, GET, ~"/a12/b34/c/d", ~[@~"a12", @~"b34/c/d"]);
        check_path(router, GET, ~"lalala", ~[]);
    }
}
