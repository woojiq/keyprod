all: build run

build:
	cargo build

run: build
	sudo target/debug/keyprod $(args)

.PHONY: all build run
