import base64::to_base64;
import elasticsearch::client;
import model::model;

iface session {
    fn session_id() -> str;

    fn user_id() -> str;
    fn set_user_id(user_id: str) -> bool;

    fn create() -> result<(str, uint), str>;

    fn delete();
}

fn mk_session(model: model) -> session {
    impl of session for model {
        fn session_id() -> str { self._id }

        fn user_id() -> str { self.get_str("user_id") }
        fn set_user_id(user_id: str) -> bool {
            self.set_str("user_id", user_id)
        }

        fn create() -> result<(str, uint), str> {
            import model::model;
            self.create()
        }

        fn save() -> result<(str, uint), str> {
            import model::model;
            self.save()
        }

        fn delete() {
            import model::model;
            self.delete()
        }
    }

    model as session
}

fn session(es: client, index: str, user_id: str) -> session {
    let id = crypto::rand::rand_bytes(32u).to_base64();
    let session = mk_session(model(es, index, "session", id));

    session.set_user_id(user_id);

    session
}

fn find(es: client, index: str, id: str) -> option<session> {
    model::find(es, index, "session", id).map { |model| mk_session(model) }
}

/*
fn handle_request(req: request::request, es: client, cookie_name: str,
                  session_index: str, user_index: str)
                  -> result<(), response::response> {
    alt req.cookies.find(cookie_name) {
      none {}
      some(cookie) {
        alt session::find(es, session_index, cookie.value) {
          none {
            // Unknown session.
            req.cookies.remove(cookie_name); 
          }
          some(session) {
            alt user::find(es, user_index, session.user_id()) {
              none {
                // No user corresponds with this session, so this session is
                // now invalid.
                session.delete();
                req.cookies.remove(cookie_name); 
              }
              some(user) {
                req.session = some(session);
                req.user = some(user);
              }
            }
          }
        }
      }
    }

    ok(())
}
*/
