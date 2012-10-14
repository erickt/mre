use std::time;
use time::Tm;
use mre::model::{Model, Error};

// Create a class to act as our model. Unfortunately Rust's classes aren't
// finished yet, and are missing a couple features that would help clean up
// models. First, there's no way to mix in implementation of functions, so we
// need to duplicate some code that's common across all models. Second, there
// is no way to have multiple constructors or static functions, so we need to
// move Error handling out into a wrapper function. So to keep the api clean,
// we cheat and hide the class so we can make a function that acts like what we
// want.
pub struct Person {
    model: Model,
}

pub impl Person {
    fn id(&self) -> &self/~str {
        &self.model._id
    }

    fn timestamp(&self) -> ~str {
        self.model.get_str(&~"timestamp")
    }

    fn set_timestamp(&mut self, timestamp: ~str) -> bool {
        self.model.set_str(~"timestamp", timestamp)
    }

    fn name(&self) -> ~str {
        self.model.get_str(&~"name")
    }

    fn set_name(&mut self, name: ~str) -> bool {
        self.model.set_str(~"name", name)
    }

    fn create(&self) -> Result<(~str, uint), Error> {
        self.model.create()
    }

    fn save(&self) -> Result<(~str, uint), Error> {
        self.model.save()
    }

    fn delete(&self) {
        self.model.delete()
    }
}

// Create a new person model.
pub fn Person(es: elasticsearch::Client, name: ~str) -> Person {
    // Create a person. We'll store the model in the ES index named
    // "helloeveryone", under the type "person". We'd like ES to make the index
    // for us, so we leave the id blank.
    let mut person = Person {
        model: Model(es, ~"helloeveryone", ~"person", ~"")
    };

    person.set_name(name);
    person.set_timestamp(time::now().rfc3339());

    person
}

// Return the last 50 people we have said hello to.
pub fn last_50(es: elasticsearch::Client) -> ~[Person] {
    // This query can be a little complicated for those who have never used
    // elasticsearch. All it says is that we want to fetch 50 documents on the
    // index "helloeveryone" and the type "person", sorted by time.
    do mre::model::search(es) |bld| {
        bld
            .set_indices(~[~"helloeveryone"])
            .set_types(~[~"person"])
            .set_source(JsonObjectBuilder()
                .insert(~"size", 50.0)
                .insert_list(~"sort", |bld| {
                    bld.push_object(|bld| {
                        bld.insert(~"timestamp", ~"desc");
                    });
                })
                .object.take()
            );
    }.map(|model|
        // Construct a person model from the raw model data.
        Person { model: *model }
    )
}
