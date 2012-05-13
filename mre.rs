import std::time;
import std::time::tm;

import request::request;
import response::response;
import middleware::middleware;

type mre = {
    m2: mongrel2::connection,
    router: router::router,
    middleware: middleware::middleware,
};

fn mre(m2: mongrel2::connection, middleware: middleware::middleware) -> mre {
    {
        m2: m2,
        router: router::router(),
        middleware: middleware,
    }
}

impl mre for mre {
    fn run() {
        loop {
            let req = self.m2.recv();
            let rep = response::response(self.m2, req);

            let req = alt request::request(req) {
              ok(req) { req }
              err(e) {
                // Ignore this request if it's malformed.
                rep.http_400(str::bytes(e));
                cont;
              }
            };

            self.middleware.wrap(req, rep);

            // Ignore close requests for now.
            if req.is_disconnect() { cont; }

            alt req.find_header("METHOD") {
              none {
                // Error out the request if we didn't get a method.
                rep.http_400(str::bytes("missing method"))
              }

              some(method) {
                alt self.router.find(method, req.path()) {
                  none { rep.http_404() }

                  some((handler, m)) { handler(req, rep, m) }
                };
              }
            };
        }
    }
}
