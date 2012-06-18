import request::request;
import response::response;

type middleware<T: copy> = fn@(@request<T>, @response) -> bool;

impl middleware<T: copy> for [middleware<T>] {
    fn wrap(req: @request<T>, rep: @response) -> bool {
        for self.each { |middleware|
            // Exit early if the middleware has handled the request.
            if !middleware(req, rep) { ret false; }
        }

        true
    }
}

fn logger<T: copy>(logger: io::writer) -> middleware<T> {
    { |req: @request<T>, rep: @response|
        let old_end = rep.end;
        rep.end = { ||
            let address = alt req.find_header("x-forwarded-for") {
              none { @"-" }
              some(address) { address }
            };

            let method = alt req.find_header("METHOD") {
              none { @"-" }
              some(method) { method }
            };

            let len = alt rep.find_header("Content-Length") {
              none { @"-" }
              some(len) { len }
            };

            logger.write_line(#fmt("%s - %s [%s] \"%s %s\" %u %s",
                *address,
                "-",
                time::now().strftime("%d/%m/%Y:%H:%M:%S %z"),
                *method,
                *req.path(),
                rep.code,
                *len));

            old_end();
        };

        true
    }
}

fn session<T: copy>(es: elasticsearch::client,
                    session_index: @str,
                    user_index: @str,
                    cookie_name: @str,
                    f: fn@(@request<T>, session::session, user::user))
  -> middleware<T> {
    { |req: @request<T>, rep: @response|
        alt req.cookies.find(*cookie_name) {
          none { }
          some(cookie) {
            alt session::find(es, session_index, cookie.value) {
              none {
                // Unknown session, so just delete the cookie.
                rep.clear_cookie(cookie_name);
              }
              some(session) {
                alt user::find(es, user_index, session.user_id()) {
                  none {
                    // We have a valid session, but no user to go with it, so
                    // delete the session and clear the cookie.
                    session.delete();
                    rep.clear_cookie(cookie_name);
                  }
                  some(user) { f(req, session, user); }
                }
              }
            }
          }
        }

        true
    }
}
