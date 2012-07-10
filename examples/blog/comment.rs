import elasticsearch::{client, delete_by_query_builder};
import mre::model::{model, error};

export comment;
export find;
export find_by_post;
export delete_by_post;

class _comment {
    let model: model;

    new(model: model) {
        self.model = model;
    }

    fn id() -> @str {
        self.model._id
    }

    fn user_id() -> @str {
        self.model.get_str("user_id")
    }

    fn set_user_id(user_id: @str) -> bool {
        self.model.set_str("user_id", user_id)
    }

    fn body() -> @str {
        self.model.get_str("body")
    }

    fn set_body(body: @str) -> bool {
        self.model.set_str("body", body)
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

type comment = _comment;

fn comment(es: client, post_id: @str, id: @str) -> comment {
    let model = model(es, @"blog", @"comment", id);
    model._parent = some(post_id);
    _comment(model)
}

fn find(es: client, post_id: @str, id: @str) -> option<comment> {
    do mre::model::find(es, @"blog", @"comment", id).map |model| {
        // Searching doesn't include the parent link, so manually add it
        // back.
        model._parent = some(post_id);
        _comment(model)
    }
}

fn find_by_post(es: client, post_id: @str) -> ~[comment] {
    do mre::model::search(es) |bld| {
        bld
            .set_indices(~["blog"])
            .set_types(~["comment"])
            .set_source(*json_dict_builder()
                .insert_dict("filter", |bld| {
                    bld.insert_dict("term", |bld| {
                        bld.insert("_parent", post_id);
                    });
                })
            );
    }.map(|model| {
        model._parent = some(post_id);
        _comment(model)
    })
}

fn delete_by_post(es: client, post_id: @str) {
    es.prepare_delete_by_query()
        .set_indices(~["blog"])
        .set_types(~["comment"])
        .set_source(*json_dict_builder()
            .insert_dict("term", |bld| {
                bld.insert("_parent", post_id);
            })
        ).execute();
}
