import base64::to_base64;
import elasticsearch::client;
import model::{model, error};

iface session {
    fn id() -> @str;

    fn user_id() -> @str;
    fn set_user_id(user_id: @str) -> bool;

    fn create() -> result<(), error>;
    fn save() -> result<(), error>;

    fn delete();
}

fn mk_session(model: model) -> session {
    impl of session for model {
        fn id() -> @str { self._id }

        fn user_id() -> @str { self.get_str("user_id") }
        fn set_user_id(user_id: @str) -> bool {
            self.set_str("user_id", user_id)
        }

        fn create() -> result<(), error> {
            import model::model;
            self.create()
        }

        fn save() -> result<(), error> {
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

fn session(es: client, index: @str, user_id: @str) -> session {
    let id = crypto::rand::rand_bytes(32u).to_base64();
    let session = mk_session(model(es, index, @"session", @id));

    session.set_user_id(user_id);

    session
}

fn find(es: client, index: @str, id: @str) -> option<session> {
    model::find(es, index, @"session", id).map { |model| mk_session(model) }
}
