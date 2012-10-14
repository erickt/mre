use request::Request;
use response::Response;

type Middleware<T: Copy> = fn@(@Request<T>, @Response) -> bool;

trait MiddlewareVec<T> {
    fn wrap(req: @Request<T>, rep: @Response) -> bool;
}

impl<T: Copy> ~[Middleware<T>]: MiddlewareVec<T> {
    fn wrap(req: @Request<T>, rep: @Response) -> bool {
        for self.each |middleware| {
            // Exit early if the middleware has handled the request.
            if !(*middleware)(req, rep) { return false; }
        }

        true
    }
}

fn logger<T: Copy>(logger: io::Writer) {
    |req: @Request<T>, rep: @Response| {
        do rep.add_end_hook || {
        let address = match req.find_header(&~"x-forwarded-for") {
          None => ~"-",
          Some(address) => copy address,
        };

        let method = match req.find_header(&~"METHOD") {
          None => ~"-",
          Some(method) => copy method,
        };

        let len = match rep.find_header(&~"Content-Length") {
          None => ~"-",
          Some(len) => copy len,
        };

        logger.write_line(#fmt("%s - %s [%s] \"%s %s\" %u %s",
            address,
            "-",
            time::now().strftime("%d/%m/%Y:%H:%M:%S %z"),
            method,
            req.req.path,
            rep.code,
            len));

        Ok(true)
        }
    };
}

/*
fn session<T: Copy>(es: elasticsearch::Client,
                    session_index: @str,
                    user_index: @str,
                    cookie_name: @str,
                    f: fn@(@Request<T>, session::session, user::user))
  -> middleware<T> {
    |req: @Request<T>, rep: @Response| {
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
*/
