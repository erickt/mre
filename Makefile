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
	cargo install mongrel2
	cargo install elasticsearch
	cargo install mustache
	cargo install pcre
	cargo install crypto

clean:
	rm -rf mre *.so *.dylib *.dSYM
