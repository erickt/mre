import std::json::json;
import elasticsearch::client;
import model::{model, error};
import auth::hasher;

export user;
export find;
export all;

class _user {
    let model: model;

    new(model: model) {
        self.model = model;
    }

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

    fn create() -> result<(), error> {
        self.model.create()
    }

    fn save() -> result<(), error> {
        self.model.save()
    }

    fn delete() {
        import model::model;
        self.model.delete()
    }
}

type user = _user;

fn user(es: client, hasher: hasher, index: @str,
        username: @str, password: @str) -> user {
    let user = _user(model(es, index, @"user", username));

    user.set_password(hasher, password);

    user
}

fn find(es: client, index: @str, id: @str) -> option<user> {
    model::find(es, index, @"user", id).map { |model| _user(model) }
}

fn all(es: client, index: @str) -> [user] {
    model::all(es, index, @"user").map { |model| _user(model) }
}
