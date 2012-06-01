Meal, Ready-to-Eat: A Web framework for the [Rust Programming
Language](http://rust-lang.org), built with
[Mongrel2](http://mongrel2.org) and
[Elasticsearch](http://elasticsearch.org).

Installation
------------

Install for users of MRE:

    % cargo install zmq
    % cargo install tnetstring
    % cargo install mongrel2
    % cargo install uri
    % cargo install elasticsearch
    % cargo install mustache
    % cargo install pcre
    % cargo install git://github.com/erickt/rustcrypto.git
    % cargo install mre

Install for developers:

    % git clone https://github.com/erickt/mre
    % cd mre
    % make deps
    % make

    % If you want to run the tests
    % make test && ./mre

Running the Hello World App
---------------------------

First, make sure [Mongrel2 is
installed](http://mongrel2.org/wiki/quick_start.html). Next, initialize
and start Mongrel2:

    % cd examples/helloworld
    % m2sh load --db config.sqlite --config mongrel2.conf
    % m2sh start -host localhost

In another shell, build and run the example:

    % make example-helloworld
    % cd ./examples/helloworld && ./helloworld

Running the Blog App
--------------------

First off, you'll need to follow the installation and setup instructions
for [rust-elasticsearch](https://github.com/erickt/rust-elasticsearch).
Once that is running, initialize and start up Mongrel2 just like before:

    % cd examples/blog
    % m2sh load --db config.sqlite --config mongrel2.conf
    % m2sh start -host localhost

In another shell, create the Elasticsearch index:

    % ./examples/blog/create-index

Finally, build and run the blog:

    % make example-blog
    % cd examples/blog && ./blog
