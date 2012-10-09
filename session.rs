use elasticsearch::Client;
use model::{model, error};

pub struct session {
    model: model,
}

impl session {
    fn id() -> @str { self.model._id }

    fn user_id() -> @str { self.model.get_str("user_id") }
    fn set_user_id(user_id: @str) -> bool {
        self.model.set_str("user_id", user_id)
    }

    fn create() -> Result<(), error> {
        self.model.create()
    }

    fn save() -> Result<(), error> {
        self.model.save()
    }

    fn delete() {
        self.model.delete()
    }
}

pub fn session(es: Client, index: @str, user_id: @str) -> session {
    let id = crypto::rand::rand_bytes(32u).to_base64();
    let session = session { model: model(es, index, @"session", @id) };

    session.set_user_id(user_id);

    session
}

pub fn find(es: Client, index: @str, id: @str) -> Option<session> {
    do model::find(es, index, @"session", id).map |model| {
        session { model: model }
    }
}
