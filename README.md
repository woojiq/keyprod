# KEY PRODuctivity

Track keyboard productivity.

## Disclaimer

I know it's not secured, but [who's gonna stop me](https://www.youtube.com/watch?v=CcG0WpGBPcY).

## TODO
- [ ] Benchmark two solutions (old and new). Use uinput to emulate input: https://www.kernel.org/doc/html/v4.12/input/uinput.html
- [ ] Rework README and make more configuration for plugins.
- [ ] Remove all unwrap and expect
- [ ] Proper logging/stdout/err
- [ ] Unit tests

## Configuration

* STATE_DIR: environment variable during compilation to set the base directory for plugin states. Default: `/var/lib/keyprod/`.

# Design

## Architecture

* Event listener: listens for the events from linux and sends them via async channel.
* Publisher: accepts events from the Event listener and sends events all messages to plugins (separately, multiple mpsc).
* Plugin runtime: single async thread for all plugins. Each plugin receives messages from the Publisher.

# Future design: plugin system

Our Event listener calls every plugin in some directory.
Plugins can be developed separately and be dynamically pluged-in.
