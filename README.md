# KEY PRODuctivity

Track keyboard productivity.

## Disclaimer

I know it's not secured, but [who's gonna stop me](https://www.youtube.com/watch?v=CcG0WpGBPcY).

I know it's overengineering to create "plugin" system for the problem that could be solved using 200loc project. But who cares. Programming is about fun.

## TODO
- [ ] Shutdown without additional keypress
- [ ] Rework kdb_event_listener
- [ ] Unit tests: https://jorgeortiz.dev/posts/rust_unit_testing_file_reading/
- [ ] Benchmark two solutions (old and new). Use uinput to emulate input: https://www.kernel.org/doc/html/v4.12/input/uinput.html
- [ ] Rewrite build.rs
- [ ] Bump version

## Configuration

For more information about all possible options run: `keyprod --help`.

There are two available plugins at the moment:
* echo (plugins/echo.rs): prints keypress events to a destination (by default `stdout`).
* history (plugins/history.rs): saves the number of keypresses made in each day to the db. Only the **total number** of keystrokes for each day is stored, there is no separate number for each key.

Examples:
```sh
# Listens for keyboard events in real time and prints to stdout.
keyprod --plugin echo

# Listens for keyboard events and stores the number of keypresses to the db.
keyprod --plugin history

# Combination of previous two commands with redefined path for the db.
keyprod --plugin echo stderr --plugin history --db-path="./history.db"

# Print all history statistics.
keyprod --plugin stat --csv
```

Log level can be configured using `RUST_LOG` environment variable. Default: info.

```sh
# Only error logs are printed.
RUST_LOG=error keyprod --plugin echo

# Logs with debug verbosity and higher are printed.
RUST_LOG=debug keyprod --plugin echo

# Disable logging.
RUST_LOG=off keyprod --plugin echo
```

# Application Design

## Architecture

* Event listener: listens for the events from linux and sends them via async channel.
* Publisher: accepts events from the Event listener and sends events all messages to plugins (separately, multiple mpsc).
* Plugin runtime: single async thread for all plugins. Each plugin receives messages from the Publisher.

# Future design: plugin system

Our Event listener calls every plugin in some directory.
Plugins can be developed separately and be dynamically pluged-in.
