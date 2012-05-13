import mre::mre;
import mre::response::response;
import zmq::context;

type app = {
    zmq: zmq::context,
    m2: mongrel2::connection,
    mre: mre::mre,
    es: elasticsearch::client,
    mu: mustache::context,
    password_hasher: mre::auth::hasher
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

    let middleware = mre::middleware::middleware([
        mre::middleware::logger(io::stdout())
    ]);

    let mre = mre::mre(m2, middleware);

    let es = elasticsearch::connect_with_zmq(zmq, "tcp://localhost:9700");

    let mu = mustache::context("views", ".mustache");

    {
        zmq: zmq,
        m2: m2,
        mre: mre,
        es: es,
        mu: mu,
        password_hasher: mre::auth::default_pbkdf2_sha1()
    }
}

impl app for app {
    fn get(regex: str, f: mre::router::handler) {
        self.mre.router.add("GET", regex, f)
    }

    fn post(regex: str, f: mre::router::handler) {
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
