use elasticsearch::Client;
use model::{Model, Error};
use auth::hasher;

pub struct User {
    model: Model,
}

pub impl User {
    fn id(&self) -> ~str { copy self.model._id }

    fn password(&self) -> ~str {
        self.model.get_str(&~"password")
    }

    fn set_password(&mut self, hasher: hasher, password: ~str) -> bool {
        let password = auth::password(hasher, password);
        self.model.set_str(~"password", password)
    }

    fn verify_password(&self, hasher: hasher, password: ~str) -> bool {
        hasher.verify(password, self.password())
    }

    fn create(&self) -> Result<(), Error> {
        self.model.create()
    }

    fn save(&self) -> Result<(), Error> {
        self.model.save()
    }

    fn delete(&self) {
        use model::Model;
        self.model.delete()
    }
}

pub fn User(es: Client, hasher: hasher, index: ~str,
        username: ~str, password: ~str) -> User {
    let mut user = User { model: Model(es, index, ~"user", username) };

    user.set_password(hasher, password);

    user
}

pub fn find(es: Client, index: ~str, id: ~str) -> Option<User> {
    model::find(es, index, ~"user", id).map(|model| User { model: *model })
}

pub fn all(es: Client, index: ~str) -> ~[User] {
    model::all(es, index, ~"user").map(|model| User { model: *model })
}
