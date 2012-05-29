fn mk_mre() -> mre::mre<()> {
    // First write some boilerplate code to create an MRE instance. This
    // code really should be abstracted away.
    let zmq = alt zmq::init(1) {
      ok(ctx) { ctx }
      err(e) { fail e.to_str() }
    };

    let m2 = mongrel2::connect(zmq,
        "E4B7CE14-E7F7-43EE-A3E6-DB7B0A0C106F",
        ["tcp://127.0.0.1:9996"],
        ["tcp://127.0.0.1:9997"]);

    // Create our middleware, which preproceses requests and responses.
    // For now we'll just use the logger.
    let mw = mre::middleware::middleware([
        mre::middleware::logger(io::stdout())
    ]);

    // Create our MRE instance. Our middleware may needs to store
    // middleware somewhere, so we need to pass in a function that creates
    // a fresh per-request data. However, since the logger middleware
    // doesn't actually need to store anything, we'll just return a unit
    // value.
    mre::mre(m2, mw) { || () }
}

fn main() {
    let mre = mk_mre();

    // Route our responses.
    mre.router.add(GET, "^/$") { |_req, rep, _m|
        rep.reply_html(200u, "
            <html>
            <body>
            <h1>Hello world!</h1>
            </body>
            </html>")
    }

    mre.run();
}
