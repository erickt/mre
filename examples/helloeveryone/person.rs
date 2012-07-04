import std::time;
import time::tm;
import mre::model::{model, error};

// Create a class to act as our model. Unfortunately Rust's classes aren't
// finished yet, and are missing a couple features that would help clean up
// models. First, there's no way to mix in implementation of functions, so we
// need to duplicate some code that's common across all models. Second, there
// is no way to have multiple constructors or static functions, so we need to
// move error handling out into a wrapper function. So to keep the api clean,
// we cheat and hide the class so we can make a function that acts like what we
// want.
class _person {
    let model: model;

    new(model: model) {
        self.model = model;
    }

    fn id() -> @str {
        self.model._id
    }

    fn timestamp() -> @str {
        self.model.get_str("timestamp")
    }

    fn set_timestamp(timestamp: @str) -> bool {
        self.model.set_str("timestamp", timestamp)
    }

    fn name() -> @str {
        self.model.get_str("name")
    }

    fn set_name(name: @str) -> bool {
        self.model.set_str("name", name)
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

// We want the type named "person", but we don't "person" to be the built-in
// class constructor.
type person = _person;

// Create a new person model.
fn person(es: client, name: @str) -> person {
    // Create a person. We'll store the model in the ES index named
    // "helloeveryone", under the type "person". We'd like ES to make the index
    // for us, so we leave the id blank.
    let person = _person(model(es, @"helloeveryone", @"person", @""));

    person.set_name(name);
    person.set_timestamp(@time::now().rfc3339());

    person
}

// Return the last 50 people we have said hello to.
fn last_50(es: client) -> ~[person] {
    // This query can be a little complicated for those who have never used
    // elasticsearch. All it says is that we want to fetch 50 documents on the
    // index "helloeveryone" and the type "person", sorted by time.
    do model::search(es) |bld| {
        bld
            .set_indices(~["helloeveryone"])
            .set_types(~["person"])
            .set_source(*json_dict_builder()
                .insert("size", 50.0)
                .insert_list("sort", |bld| {
                    bld.push_dict(|bld| {
                        bld.insert("timestamp", "desc");
                    });
                })
            );
    }.map(|model|
        // Construct a person model from the raw model data.
        _person(model)
    )
}
