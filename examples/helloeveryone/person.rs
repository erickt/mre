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
    // We'll let ES come up with a unique id.
    let person = _person(model(es, @"helloeveryone", @"person", @""));

    person.set_name(name);
    person.set_timestamp(@time::now().rfc3339());

    person
}

// Return the last 100 people we have said hello to.
fn last_50(es: client) -> [person] {
    // Sort by reverse timestamp.
    model::search(es) { |bld|
        bld
            .set_indices(["helloeveryone"])
            .set_types(["person"])
            .set_source(*json_dict_builder()
                .insert("size", 50.0)
                .insert_list("sort") { |bld|
                    bld.push_dict { |bld|
                        bld.insert("timestamp", "desc");
                    };
                }
            );
    }.map { |model| _person(model) }
}
