import person::person;

fn main() {
    // Create a zeromq context that MRE will use to talk to Mongrel2 and
    // Elasticsearch.
    let zmq = alt zmq::init(1) {
        ok(ctx) { ctx }
        err(e) { fail e.to_str() }
    };

    let mre = mre::mre(
        zmq,

        // Generate a new UUID each time we start.
        none,

        // The addresses to receive requests from.
        ~["tcp://127.0.0.1:9994"],

        // The addresses to send responses to.
        ~["tcp://127.0.0.1:9995"],

        // Create our middleware, which preproceses requests and
        // responses. For now we'll just use the logger.
        ~[mre::middleware::logger(io::stdout())],

        // A function to create per-request data. This can be used by
        // middleware like middleware::session to automatically look
        // up the current user and session data in the database. We don't
        // need it for this example, so just return a unit value.
        || ()
    );

    // Connect to Elasticsearch, which we'll use as our database.
    let es = elasticsearch::connect_with_zmq(zmq, "tcp://localhost:9700");

    // Show who we'll say hello to.
    do mre.get("^/$") |_req, rep, _m| {
        // Fetch the people we've greeted.
        let people = person::last_50(es);

        // We want to render out our responses using mustache, so we need
        // to convert our model over to something mustache can handle.
        let template = mustache::render_file("index", hash_from_strs(~[
            ("names", do people.map |person| {
                hash_from_strs(~[
                    ("name", person.name())
                ])
            }.to_mustache())
        ]));
               
        rep.reply_html(200u, template)
    }

    // Add a new person to greet.
    do mre.post("^/$") |req, rep, _m| {
        // Parse the form data.
        let form = uri::decode_form_urlencoded(*req.body());

        alt form.find("name") {
          none {
            rep.reply_http(400u, "missing name");
          }
          some(names) {
            // Create and save our person. If successful, redirect back to
            // the front page.
            let person = person::person(es, (*names)[0u]);

            alt person.create() {
              ok(()) { rep.reply_redirect("/") }
              err(e) {
                // Uh oh, something bad happened. Let's just display the
                // error back to the user for now.
                rep.reply_http(500u, e.msg)
              }
            }
          }
        }
    }

    // Finally, start the MRE event loop.
    mre.run();
}
