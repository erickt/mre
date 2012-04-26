import std::json::json;
import elasticsearch::client;
import model::model;
import auth::hasher;

export user;
export find;
export all;

iface user {
    fn user_id() -> str;

    fn password() -> str;
    fn set_password(hasher: hasher, password: str) -> bool;
    fn verify_password(hasher: hasher, password: str) -> bool;

    fn create() -> result<(str, uint), str>;
    fn save() -> result<(str, uint), str>;

    fn delete();
}

fn mk_user(model: model) -> user {
    impl of user for model {
        fn user_id() -> str {
            self._id
        }

        fn password() -> str {
            self.get_str("password")
        }

        fn set_password(hasher: hasher, password: str) -> bool {
            let password = auth::password(hasher, password);
            self.set_str("password", password)
        }

        fn verify_password(hasher: hasher, password: str) -> bool {
            hasher.verify(password, self.password())
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

    model as user
}

fn user(es: client, hasher: hasher, index: str,
        username: str, password: str) -> user {
    let user = mk_user(model(es, index, "user", username));

    user.set_password(hasher, password);

    user
}

fn find(es: client, index: str, id: str) -> option<user> {
    model::find(es, index, "user", id).map { |model| mk_user(model) }
}

fn all(es: client, index: str) -> [user] {
    model::all(es, index, "user").map { |model| mk_user(model) }
}
