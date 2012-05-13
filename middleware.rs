import request::request;
import response::response;

type wrapper = fn@(@request, @response);

type middleware = {
    wrappers: [wrapper],
};

fn middleware(wrappers: [wrapper]) -> middleware {
    { wrappers: wrappers }
}

impl middleware for middleware {
    fn wrap(req: @request, rep: @response) {
        self.wrappers.iter { |wrapper| wrapper(req, rep); }
    }
}

fn logger(logger: io::writer) -> wrapper {
    { |req: @request, rep: @response|
        let old_end = rep.end;
        rep.end = { ||
            let address = alt req.find_header("x-forwarded-for") {
              none { "-" }
              some(address) { address }
            };

            let method = alt req.find_header("METHOD") {
              none { "-" }
              some(method) { method }
            };

            let len = alt rep.find_header("Content-Length") {
              none { "-" }
              some(len) { len }
            };

            logger.write_line(#fmt("%s - %s [%s] \"%s %s\" %u %s",
                address,
                "-",
                time::now().strftime("%d/%m/%Y:%H:%M:%S %z"),
                method,
                req.path(),
                rep.code,
                len));

            old_end();
        };
    }
}
