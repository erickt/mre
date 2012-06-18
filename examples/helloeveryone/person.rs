import std::time;
import time::tm;
import mre::model::{model, error};

// Create an interface to act as our model. We do this to protect against
// accidentally passing one raw model to another. We want that to be a
// type error, so we wrap the raw models in an interface, and hide the
// implementation inside a function.
iface person {
    fn id() -> @str;

    fn timestamp() -> @str;
    fn set_timestamp(timestamp: @str) -> bool;

    fn name() -> @str;
    fn set_name(name: @str) -> bool;

    fn create() -> result<(), error>;
    fn save() -> result<(), error>;
    fn delete();
}

// This function takes a raw model and hides the implementation of the
// person interface.
fn mk_person(model: model) -> person {
    impl of person for model {
        fn id() -> @str {
            self._id
        }

        fn timestamp() -> @str {
            self.get_str("timestamp")
        }

        fn set_timestamp(timestamp: @str) -> bool {
            self.set_str("timestamp", timestamp)
        }

        fn name() -> @str {
            self.get_str("name")
        }

        fn set_name(name: @str) -> bool {
            self.set_str("name", name)
        }

        fn create() -> result<(), error> {
            import model::model;
            self.create()
        }

        fn save() -> result<(), error> {
            import model::model;
            self.save()
        }

        fn delete() {
            import model::model;
            self.delete()
        }
    }

    model as person
}

// Create a new person model.
fn person(es: client, name: @str) -> person {
    // We'll let ES come up with a unique id.
    let person = mk_person(model(es, @"helloeveryone", @"person", @""));

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
    }.map { |model| mk_person(model) }
}
