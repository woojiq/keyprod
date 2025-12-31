# KEY PRODuctivity

Track keyboard productivity.

## Disclaimer

I know it's not secured, but [who's gonna stop me](https://www.youtube.com/watch?v=CcG0WpGBPcY).

## Development

To run debug build without sudo, add your user to "input" group.

```bash
# Build and run locally
make
```

## Architecture

* Event listener: listens for the events from linux and sends them via async channel.
* Publisher: accepts subscriptions, listens async channel and calls all callbacks.
* Subscribers: subscribes for key events.

# Future design: plugin system

Our Event listener calls every plugin in some directory.
Plugins can be developed separately and be dynamically pluged-in.
