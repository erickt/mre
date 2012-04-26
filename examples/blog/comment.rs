import elasticsearch::{client, delete_by_query_builder};
import mre::model::model;

export comment;
export find;
export find_by_post;
export delete_by_post;

iface comment {
    fn comment_id() -> str;

    fn user_id() -> str;
    fn set_user_id(user_id: str) -> bool;

    fn body() -> str;
    fn set_body(body: str) -> bool;

    fn create() -> result<(str, uint), str>;
    fn save() -> result<(str, uint), str>;

    fn delete();
}

fn mk_comment(model: model) -> comment {
    impl of comment for model {
        fn comment_id() -> str {
            self._id
        }

        fn user_id() -> str {
            self.get_str("user_id")
        }

        fn set_user_id(user_id: str) -> bool {
            self.set_str("user_id", user_id)
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
    }

    model as comment
}

fn comment(es: client, post_id: str, id: str) -> comment {
    let model = model(es, "blog", "comment", id);
    let model = { _parent: some(post_id) with model };
    mk_comment(model)
}

fn find(es: client, post_id: str, id: str) -> option<comment> {
    model::find(es, "blog", "comment", id).map { |model|
        mk_comment({ _parent: some(post_id) with model })
    }
}

fn find_by_post(es: client, post_id: str) -> [comment] {
    model::search(es) { |bld|
        bld
            .set_indices(["blog"])
            .set_types(["comment"])
            .set_source(*json_dict_builder()
                .insert_dict("filter") { |bld|
                    bld.insert_dict("term") { |bld|
                        bld.insert_str("_parent", post_id);
                    };
                }
            );
    }.map { |model|
        mk_comment({ _parent: some(post_id) with model })
    }
}

fn delete_by_post(es: client, post_id: str) {
    let rep = es.prepare_delete_by_query()
        .set_indices(["blog"])
        .set_types(["comment"])
        .set_source(*json_dict_builder()
            .insert_dict("term") { |bld|
                bld.insert_str("_parent", post_id);
            }
        ).execute();

    #error("%?", rep);
}
