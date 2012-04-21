import mre::mre;
import mre::response::response;
import zmq::context;

type app = {
    zmq: zmq::context,
    m2: mongrel2::connection,
    mre: mre::mre,
    es: elasticsearch::client,
    mu: mustache::context
};

fn app() -> app {
    let zmq =
        alt zmq::init(1) {
          ok(ctx) { ctx }
          err(e) { fail e.to_str() }
        };

    let m2 = mongrel2::connect(zmq,
        "F0D32575-2ABB-4957-BC8B-12DAC8AFF13A",
        "tcp://127.0.0.1:9998",
        "tcp://127.0.0.1:9999");

    let mre = mre::mre(m2, io::stdout());

    let es = elasticsearch::connect_with_zmq(zmq, "tcp://localhost:9700");

    let mu = mustache::context("views", ".mustache");

    { zmq: zmq, m2: m2, mre: mre, es: es, mu: mu }
}

impl app for app {
    fn get(regex: str, f: fn@(request, pcre::match) -> mre::response::response) {
        self.mre.router.add("GET", regex, f)
    }

    fn post(regex: str, f: fn@(request, pcre::match) -> mre::response::response) {
        self.mre.router.add("POST", regex, f)
    }

    fn run() {
        self.mre.run();
    }

    fn term() {
        self.m2.term();
        self.zmq.term();
    }
}
