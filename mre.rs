import std::time;
import std::time::tm;
import m2_request = mongrel2::request;
import zmq::error;

import router::router;
import request::request;
import response::response;
import middleware::middleware;

import to_bytes::to_bytes;

type mre<T: copy> = @{
    m2: mongrel2::connection,
    router: router::router<T>,
    middleware: [middleware<T>],
    data: fn@() -> T,
};

#[doc = "
Helper function to abstract away some of the boilerplate code.
"]
fn mre<T: copy>(zmq: zmq::context,
                +sender_id: option<str>,
                +req_addrs: [str],
                +rep_addrs: [str],
                middleware: [middleware<T>],
                data: fn@() -> T) -> mre<T> {  
    @{
        m2: mongrel2::connect(zmq, sender_id, req_addrs, rep_addrs),
        router: router::router(),
        middleware: copy middleware,
        data: data
    }
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

            let req = alt request::request(m2_req, rep, self.data()) {
              none {
                // Ignore this request if it's malformed.
                cont;
              }
              some(req) { req }
            };

            if self.middleware.wrap(req, rep) {
                // Only run the handler if the middleware hasn't handled
                // the request.
                alt self.router.find(req.method, *req.path()) {
                  none { rep.reply_http(404u, "") }
                  some((handler, m)) { handler(req, rep, m) }
                };
            }
        }
    }

    fn head(regex: str, f: router::handler<T>) {
        self.router.add(request::HEAD, regex, f)
    }

    fn get(regex: str, f: router::handler<T>) {
        self.router.add(request::GET, regex, f)
    }

    fn post(regex: str, f: router::handler<T>) {
        self.router.add(request::POST, regex, f)
    }

    fn put(regex: str, f: router::handler<T>) {
        self.router.add(request::PUT, regex, f)
    }

    fn delete(regex: str, f: router::handler<T>) {
        self.router.add(request::DELETE, regex, f)
    }

    fn trace(regex: str, f: router::handler<T>) {
        self.router.add(request::TRACE, regex, f)
    }

    fn options(regex: str, f: router::handler<T>) {
        self.router.add(request::OPTIONS, regex, f)
    }

    fn connect(regex: str, f: router::handler<T>) {
        self.router.add(request::CONNECT, regex, f)
    }

    fn patch(regex: str, f: router::handler<T>) {
        self.router.add(request::PATCH, regex, f)
    }
}
