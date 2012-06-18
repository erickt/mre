import mre::mre;
import mre::router::router;
import mre::response::response;
import zmq::context;

type data = @{
    mut session: option<mre::session::session>,
    mut user: option<mre::user::user>,
};

type app = @{
    zmq: zmq::context,
    mre: mre::mre<data>,
    es: elasticsearch::client,
    mu: mustache::context,
    password_hasher: mre::auth::hasher
};

fn app() -> app {
    let zmq = alt zmq::init(1) {
      ok(ctx) { ctx }
      err(e) { fail e.to_str() }
    };

    // Connect to Elasticsearch, which we'll use as our database.
    let es = elasticsearch::connect_with_zmq(zmq, "tcp://localhost:9700");

    // Create our middleware. We'll use the session middleware so we can
    // automatically log a user in based off a session cookie.
    let middleware = [
        mre::middleware::logger(io::stdout()),
        mre::middleware::session(es,
            @"blog",
            @"blog",
            @"session"
        ) { |req: @request<data>, session, user|
            req.data.session = some(session);
            req.data.user = some(user);
        }
    ];

    // 
    let mre = mre::mre(zmq,
        some("F0D32575-2ABB-4957-BC8B-12DAC8AFF13A"),
        ["tcp://127.0.0.1:9998"],
        ["tcp://127.0.0.1:9999"],
        middleware,
        { || @{ mut session: none, mut user: none } });

    @{
        zmq: zmq,
        mre: mre,
        es: es,

        // We store our mustache views in a subdirectory, so we need to
        // make our own context to tell mustache where to look.
        mu: mustache::context("views", ".mustache"),

        // Create a password hasher that uses the pbkdf2 algorithm.
        password_hasher: mre::auth::default_pbkdf2_sha1()
    }
}

impl app for app {
    fn get(regex: str, f: mre::router::handler<data>) {
        self.mre.get(regex, f)
    }

    fn post(regex: str, f: mre::router::handler<data>) {
        self.mre.post(regex, f)
    }

    fn run() {
        self.mre.run();
    }

    fn term() {
        self.mre.m2.term();
        self.zmq.term();
    }
}
