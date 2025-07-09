# Design

## Plan

Database: sqlite

1. Store only total number of keypresses for each day.
2.
  1. On SIGTERM save in-memory statisticks to database.
  2. When the new day begins - save previous statisticks to database and clear in-memory data.
3. Make systemd service which must be started after login (for security concerns).

This simple utility is not responsible for displaying data in "human" readable format. It will be done by another utility.

## Database scheme

Database name: keyprod.db
Table name: KeyPresses
Scheme:
  Data - Number of keypresses

## Architecture

* Event listener: listens for the events from linux and sends them via async channel.
* Publisher: accepts subscriptions, listens async channel and calls all callbacks.
* Subscribers: subscribes for key events.
