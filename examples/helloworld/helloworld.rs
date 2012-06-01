fn main() {
    let mre = mre::mre(
        // A UUID for this Mongrel2 backend.
        "E4B7CE14-E7F7-43EE-A3E6-DB7B0A0C106F",

        // The addresses to receive requests from.
        ["tcp://127.0.0.1:9996"],

        // The addresses to send responses to.
        ["tcp://127.0.0.1:9997"],

        // Create our middleware, which preproceses requests and
        // responses. For now we'll just use the logger.
        [mre::middleware::logger(io::stdout())],

        // A function to create per-request data. This can be used by
        // middleware like middleware::session to automatically look
        // up the current user and session data in the database. We don't
        // need it for this example, so just return a unit value.
        { || () }
    );

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
