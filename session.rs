import elasticsearch::client;
import model::{model, error};

class _session {
    let model: model;

    new(model: model) {
        self.model = model;
    }
    
    fn id() -> @str { self.model._id }

    fn user_id() -> @str { self.model.get_str("user_id") }
    fn set_user_id(user_id: @str) -> bool {
        self.model.set_str("user_id", user_id)
    }

    fn create() -> result<(), error> {
        self.model.create()
    }

    fn save() -> result<(), error> {
        self.model.save()
    }

    fn delete() {
        self.model.delete()
    }
}

type session = _session;

fn session(es: client, index: @str, user_id: @str) -> session {
    let id = crypto::rand::rand_bytes(32u).to_base64();
    let session = _session(model(es, index, @"session", @id));

    session.set_user_id(user_id);

    session
}

fn find(es: client, index: @str, id: @str) -> option<session> {
    model::find(es, index, @"session", id).map(|model| _session(model))
}
