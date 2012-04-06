import io::{writer, writer_util};
import std::map::hashmap;
import std::time;
import std::time::tm;

type mre = {
    m2: mongrel2::connection,
    router: router::router,
    logger: io::writer,
};

fn mre(m2: mongrel2::connection, logger: io::writer) -> mre {
    { m2: m2, router: router::router(), logger: logger }
}

impl mre for mre {
    fn run() {
        loop {
            let req = self.m2.recv();

            let rep = alt req.headers.find("METHOD") {
              none {
                // Error out the request if we didn't get a method.
                response::http_400(req)
              }

              some(methods) {
                let rep = alt self.router.find(methods[0], req.path) {
                  none { response::http_404(req) }
                  some((handler, m)) { handler(req, m) }
                };

                rep
              }
            };

            self.log_response(rep);

            self.m2.reply_http(
                rep.request,
                rep.code,
                rep.status,
                rep.headers,
                rep.body);
        }
    }

    fn log_response(rep: response::response) {
        let req = rep.request;

        let address = alt req.headers.find("x-forwarded-for") {
          none { "-" }
          some(addresses) { addresses[0] }
        };

        let method = alt req.headers.find("METHOD") {
          none { "-" }
          some(methods) { methods[0] }
        };

        let size = rep.body.len();

        self.logger.write_line(#fmt("%s - %s [%s] \"%s %s\" %u %s",
            address,
            "-",
            time::now().strftime("%d/%m/%Y:%H:%M:%S %z"),
            method,
            req.path,
            rep.code,
            if size == 0u { "-" } else { #fmt("%u", size) }));
    }
}
