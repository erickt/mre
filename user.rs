use elasticsearch::Client;
use model::{model, error};
use auth::hasher;

export user;
export find;
export all;

pub struct user {
    model: model,
}

impl user {
    fn id() -> @str { self.model._id }

    fn password() -> @str {
        self.model.get_str("password")
    }

    fn set_password(hasher: hasher, password: @str) -> bool {
        let password = auth::password(hasher, password);
        self.model.set_str("password", @password)
    }

    fn verify_password(hasher: hasher, password: @str) -> bool {
        hasher.verify(password, self.password())
    }

    fn create() -> Result<(), error> {
        self.model.create()
    }

    fn save() -> Result<(), error> {
        self.model.save()
    }

    fn delete() {
        use model::model;
        self.model.delete()
    }
}

pub fn user(es: Client, hasher: hasher, index: @str,
        username: @str, password: @str) -> user {
    let user = user { model: model(es, index, @"user", username) };

    user.set_password(hasher, password);

    user
}

pub fn find(es: Client, index: @str, id: @str) -> Option<user> {
    model::find(es, index, @"user", id).map(|model| user { model: model })
}

pub fn all(es: Client, index: @str) -> ~[user] {
    model::all(es, index, @"user").map(|model| user { model: model })
}
