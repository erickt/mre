import elasticsearch::client;
import mre::model::{model, error};

export post;
export find;
export all;
export find_by_user;

class _post {
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

    fn title() -> @str {
        self.model.get_str("title")
    }

    fn set_title(title: @str) -> bool {
        self.model.set_str("title", title)
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

    fn find_comments() -> ~[comment::comment] {
        comment::find_by_post(self.model.es, self.model._id)
    }
}

type post = _post;

fn post(es: client, id: @str) -> post {
    _post(model(es, @"blog", @"post", id))
}

fn find(es: client, id: @str) -> option<post> {
    model::find(es, @"blog", @"post", id).map(|model| _post(model))
}

fn all(es: client) -> ~[post] {
    model::all(es, @"blog", @"post").map(|model| _post(model))
}

fn find_by_user(es: client, user_id: @str) -> ~[post] {
    do model::search(es) |bld| {
        bld
            .set_indices(~["blog"])
            .set_types(~["post"])
            .set_source(*json_dict_builder()
                .insert_dict("filter", |bld| {
                    bld.insert_dict("term", |bld| {
                        bld.insert("user_id", copy user_id);
                    });
                })
            );
    }.map(|model| _post(model))
}
