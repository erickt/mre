import std::time;
import std::time::tm;
import m2_request = mongrel2::request;

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
        io::println(#fmt("Starting up %? -> %?",
            self.m2.sub_addrs(),
            self.m2.pub_addrs()));

        loop {
            let m2_req = self.m2.recv();

            // Ignore close requests for now.
            if m2_req.is_disconnect() { cont; }

            let rep = response::response(self.m2, m2_req);

            let req = alt request::request(m2_req, rep, self.mk_data()) {
              none {
                // Ignore this request if it's malformed.
                cont;
              }
              some(req) { req }
            };

            self.middleware.wrap(req, rep);

            alt self.router.find(req.method, req.path()) {
              none { rep.http_404("") }
              some((handler, m)) { handler(req, rep, m) }
            };
        }
    }
}
