STATE_DIR = "./"

all: build run

build:
	STATE_DIR="${DB_PATH}" cargo build

run: build
	sudo target/debug/keyprod $(args)

.PHONY: all build run
