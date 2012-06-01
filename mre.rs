import std::time;
import std::time::tm;
import m2_request = mongrel2::request;
import zmq::error;

import router::router;
import request::request;
import response::response;
import middleware::middleware;

import to_bytes::to_bytes;

type mre<T> = {
    zmq: zmq::context,
    m2: mongrel2::connection,
    router: router::router<T>,
    middleware: middleware::middleware<T>,
    mk_data: fn@() -> T,
};

fn mre_builder<T: copy>(zmq: zmq::context,
                        m2: mongrel2::connection,
                        middleware: middleware::middleware<T>,
                        mk_data: fn@() -> T) -> mre<T> {
    {
        zmq: zmq,
        m2: m2,
        router: router::router(),
        middleware: middleware,
        mk_data: mk_data,
    }
}

#[doc = "
Helper function to abstract away some of the boilerplate code.
"]
fn mre<T: copy>(sender_id: str,
                req_addrs: [str],
                rep_addrs: [str],
                middleware: [middleware::wrapper<T>],
                mk_data: fn@() -> T) -> mre<T> {
    // First write some boilerplate code to create an MRE instance. This
    // code really should be abstracted away.
    let zmq = alt zmq::init(1) {
      ok(ctx) { ctx }
      err(e) { fail e.to_str() }
    };

    let m2 = mongrel2::connect(zmq, sender_id, req_addrs, rep_addrs);

    // Create our middleware, which preproceses requests and responses.
    // For now we'll just use the logger.
    let mw = middleware::middleware(middleware);

    // Create our MRE instance. Our middleware may needs to store
    // middleware somewhere, so we need to pass in a function that creates
    // a fresh per-request data. However, since the logger middleware
    // doesn't actually need to store anything, we'll just return a unit
    // value.
    mre_builder(zmq, m2, mw, mk_data)
}

impl mre<T: copy> for mre<T> {
    fn run() {
        io::println(#fmt("Starting up %? -> %?",
            self.m2.req_addrs(),
            self.m2.rep_addrs()));

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
              none { rep.reply_http(404u, "") }
              some((handler, m)) { handler(req, rep, m) }
            };
        }
    }
}
