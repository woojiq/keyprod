# KEY PRODuctivity

## Disclaimer

I know it's not secured, but [who's gonna stop me](https://www.youtube.com/watch?v=CcG0WpGBPcY).

# Design

## Plan

Database: sqlite

1. Store only total number of keypresses for each day.
2.
  1. Every minute (or 5) or on SIGTERM save in-memory statistics to database.
  2. Save data to the corresponsing day.
3. Make systemd service which must be started after login (for security concerns).

This simple utility is not responsible for displaying data in "human" readable format. It will be done by another utility.

## Database scheme

Database name: keyprod.db
Table name: KeyPresses
Scheme:
  * date text
  * key int
  * count int not null default 0
  * primary key(date, key)

## Architecture

* Event listener: listens for the events from linux and sends them via async channel.
* Publisher: accepts subscriptions, listens async channel and calls all callbacks.
* Subscribers: subscribes for key events.

# Future design: plugin system

Our Event listener calls every plugin in some directory.
Plugins can be developed separately and be dynamically pluged-in.
