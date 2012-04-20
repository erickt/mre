import io::{reader, reader_util};
import result::{ok, err, extensions};

import std::map::{hashmap, str_hash, hash_from_strs};
import std::json;

import elasticsearch::{client, search_builder, index_builder, json_dict_builder};
import mongrel2::{connection, request};
import mre::mre;
import mu_context = mustache::context;
import mustache::to_mustache;
import zmq_context = zmq::context;
import zmq::error;

import app::app;

fn main() {
    let app = app();
    routes::routes(app);
    app.run();
}
