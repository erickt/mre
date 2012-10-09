use request::request;
use response::response;

type middleware<T: Copy> = fn@(@request<T>, @response) -> bool;

impl<T: Copy> ~[middleware<T>] {
    fn wrap(req: @request<T>, rep: @response) -> bool {
        for self.each |middleware| {
            // Exit early if the middleware has handled the request.
            if !middleware(req, rep) { return false; }
        }

        true
    }
}

fn logger<T: Copy>(logger: io::Writer) -> middleware<T> {
    |req: @request<T>, rep: @response| {
        let old_end = rep.end;
        rep.end = || {
            let address = match req.find_header("x-forwarded-for") {
              None => @"-",
              Some(address) => address,
            };

            let method = match req.find_header("METHOD") {
              None => @"-",
              Some(method) => method,
            };

            let len = match rep.find_header("Content-Length") {
              None => @"-",
              Some(len) => len,
            };

            logger.write_line(#fmt("%s - %s [%s] \"%s %s\" %u %s",
                *address,
                "-",
                time::now().strftime("%d/%m/%Y:%H:%M:%S %z"),
                *method,
                *req.path(),
                rep.code,
                *len));

            old_end()
        };

        true
    }
}

fn session<T: Copy>(es: elasticsearch::Client,
                    session_index: @str,
                    user_index: @str,
                    cookie_name: @str,
                    f: fn@(@request<T>, session::session, user::user))
  -> middleware<T> {
    |req: @request<T>, rep: @response| {
        match req.cookies.find(*cookie_name) {
          None => { },
          Some(cookie) => {
            match session::find(es, session_index, cookie.value) {
              None => {
                // Unknown session, so just delete the cookie.
                rep.clear_cookie(cookie_name);
              },
              Some(session) => {
                match user::find(es, user_index, session.user_id()) {
                  None => {
                    // We have a valid session, but no user to go with it, so
                    // delete the session and clear the cookie.
                    session.delete();
                    rep.clear_cookie(cookie_name);
                  }
                  Some(user) => f(req, session, user),
                }
              }
            }
          }
        }

        true
    }
}
