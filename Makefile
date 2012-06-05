all:
	rustc mre.rc

test:
	rustc --test mre.rc

examples: example-helloworld example-helloeveryone example-blog

example-helloworld: all
	rustc -L . examples/helloworld/helloworld.rc

example-helloeveryone: all
	rustc -L . examples/helloeveryone/helloeveryone.rc

example-blog: all
	rustc -L . examples/blog/blog.rc

deps:
	cargo install -g zmq
	cargo install -g tnetstring
	cargo install -g mongrel2
	cargo install -g uri
	cargo install -g elasticsearch
	cargo install -g mustache
	cargo install -g pcre
	cargo install -g crypto

clean:
	rm -rf mre *.so *.dylib *.dSYM
