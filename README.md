Meal, Ready-to-Eat: A Web framework for the [Rust Programming
Language](http://rust-lang.org), built with
[Mongrel2](http://mongrel2.org) and
[Elasticsearch](http://elasticsearch.org).

Installation
------------

Install for users of MRE:

    % cargo install mre

Install for developers:

    % git clone https://github.com/erickt/mre
    % cd mre
    % make deps
    % make

    % If you want to run the tests
    % make test && ./mre

Running the Examples
--------------------

First, make sure [Mongrel2 is
installed](http://mongrel2.org/wiki/quick_start.html). Next, initialize
and start Mongrel2:

    % cd examples/helloworld
    % m2sh load --db config.sqlite --config mongrel2.conf
    % m2sh start -host localhost

In another shell, build and run the `Hello World` example:

    % make example-helloworld
    % cd ./examples/helloworld && ./helloworld

The `Hello Everyone` example needs a little more setup before it can run. It
uses Elasticsearch, so follow these
[instructions](https://github.com/erickt/rust-elasticsearch) to get ES running.
Then, create the example's index:

    % ./examples/helloeveryone/create-index

Start up Mongrel2:

    % cd examples/helloeveryone
    % m2sh load --db config.sqlite --config mongrel2.conf
    % m2sh start -host localhost

And in a separate shell, build and run the example:

    % make example-helloeveryone
    % cd examples/helloeveryone && ./helloeveryone

The `Blog` example is built the same way, just replace `helloeveryone` with
`blog`.
