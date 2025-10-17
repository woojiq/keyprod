DB_PATH = "./"

all: build run

build:
	DB_PATH="${DB_PATH}" cargo build

# TODO: add capabilities to binary using linux commands?
run: build
	sudo target/debug/keyprod

.PHONY: all build run
