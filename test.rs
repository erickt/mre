use std;
use zmq;
use mongrel2;
use mre;

import result::{ok, err};
import zmq::{context, error};
import mongrel2::connection;
import mre::mre;

fn main() {
    let ctx =
        alt zmq::init(1) {
          ok(ctx) { ctx }
          err(e) { fail e.to_str() }
        };

    let m2 = mongrel2::connect(ctx,
        "F0D32575-2ABB-4957-BC8B-12DAC8AFF13A",
        "tcp://127.0.0.1:9998",
        "tcp://127.0.0.1:9999");

    let mre = mre::mre(m2);
    mre.run();

    m2.term();
    ctx.term();
}
