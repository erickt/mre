import elasticsearch::client;
import mre::model::model;

export post;
export find;
export all;
export find_by_user;

iface post {
    fn post_id() -> str;

    fn user_id() -> str;
    fn set_user_id(user_id: str) -> bool;

    fn title() -> str;
    fn set_title(title: str) -> bool;

    fn body() -> str;
    fn set_body(body: str) -> bool;

    fn create() -> result<(str, uint), str>;
    fn save() -> result<(str, uint), str>;

    fn delete();

    fn find_comments() -> [comment::comment];
}

fn mk_post(model: model) -> post {
    impl of post for model {
        fn post_id() -> str {
            self._id
        }

        fn user_id() -> str {
            self.get_str("user_id")
        }

        fn set_user_id(user_id: str) -> bool {
            self.set_str("user_id", user_id)
        }

        fn title() -> str {
            self.get_str("title")
        }

        fn set_title(title: str) -> bool {
            self.set_str("title", title)
        }

        fn body() -> str {
            self.get_str("body")
        }

        fn set_body(body: str) -> bool {
            self.set_str("body", body)
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

        fn find_comments() -> [comment::comment] {
            comment::find_by_post(self.es, self._id)
        }
    }
    
    model as post
}

fn post(es: client, id: str) -> post {
    mk_post(model(es, "blog", "post", id))
}

fn find(es: client, id: str) -> option<post> {
    model::find(es, "blog", "post", id).map { |model| mk_post(model) }
}

fn all(es: client) -> [post] {
    model::all(es, "blog", "post").map { |model| mk_post(model) }
}

fn find_by_user(es: client, user_id: str) -> [post] {
    model::search(es) { |bld|
        bld
            .set_indices(["blog"])
            .set_types(["post"])
            .set_source(*json_dict_builder()
                .insert_dict("filter") { |bld|
                    bld.insert_dict("term") { |bld|
                        bld.insert_str("user_id", user_id);
                    };
                }
            );
    }.map { |model| mk_post(model) }
}
