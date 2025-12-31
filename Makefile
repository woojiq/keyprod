all: build run

build:
	cargo build

# TODO: remove sudo
run: build
	mkdir -p images
	target/debug/keyprod --state=./ --plugins echo history $(args)

.PHONY: all build run
