use elasticsearch::Client;
use model::{Model, Error};

pub struct Session {
    model: Model,
}

impl Session {
    fn id(&self) -> &self/~str { &self.model._id }

    fn user_id(&self) -> ~str { self.model.get_str(&~"user_id") }

    fn set_user_id(&mut self, user_id: ~str) -> bool {
        self.model.set_str(~"user_id", user_id)
    }

    fn create(&self) -> Result<(~str, uint), Error> {
        self.model.create()
    }

    fn save(&self) -> Result<(~str, uint), Error> {
        self.model.save()
    }

    fn delete(&self) {
        self.model.delete()
    }
}

pub fn Session(es: Client, index: ~str, user_id: ~str) -> Session {
    let id = crypto::rand::rand_bytes(32u).to_base64();
    let mut session = Session { model: Model(es, index, ~"session", id) };

    session.set_user_id(user_id);

    session
}

pub fn find(es: Client, index: ~str, id: ~str) -> Option<Session> {
    do model::find(es, index, ~"session", id).map |model| {
        Session { model: *model }
    }
}
