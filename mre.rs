import std::time;
import std::time::tm;

import router::router;
import request::request;
import response::response;
import middleware::middleware;

import to_bytes::to_bytes;

type mre<T> = {
    m2: mongrel2::connection,
    router: router::router<T>,
    middleware: middleware::middleware<T>,
    mk_data: fn@() -> T,
};

fn mre<T: copy>(m2: mongrel2::connection,
          middleware: middleware::middleware<T>,
          mk_data: fn@() -> T) -> mre<T> {
    {
        m2: m2,
        router: router::router(),
        middleware: middleware,
        mk_data: mk_data,
    }
}

impl mre<T: copy> for mre<T> {
    fn run() {
        loop {
            let req = self.m2.recv();
            let rep = response::response(self.m2, req);

            let req = alt request::request(req, self.mk_data()) {
              ok(req) { req }
              err(e) {
                // Ignore this request if it's malformed.
                rep.http_400(e);
                cont;
              }
            };

            self.middleware.wrap(req, rep);

            // Ignore close requests for now.
            if req.is_disconnect() { cont; }

            alt req.find_header("METHOD") {
              none {
                // Error out the request if we didn't get a method.
                rep.http_400("missing method")
              }

              some(method) {
                alt self.router.find(method, req.path()) {
                  none { rep.http_404("") }

                  some((handler, m)) { handler(req, rep, m) }
                };
              }
            };
        }
    }
}
