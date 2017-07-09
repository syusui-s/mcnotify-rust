mcnotify-rust: Minecraft status notifier written in Rust
===

[![LICENSE](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Build Status](https://travis-ci.org/syusui-s/mcnotify-rust.svg?branch=master)](https://travis-ci.org/syusui-s/mcnotify-rust)

**CURRENTLY UNDER DEVELOPMENT.**

## Overview
mcnotify send notifications about server status when its change.

will support these informations for notifications:

* Server stopped or recovered
* Player joined or left
* the number of players

will support these integrations:

* IFTTT Maker Channel
* (Twitter)
* (Slack)

## How to build
If you want to cross compile, see <https://blog.rust-lang.org/2016/05/13/rustup.html>.

```console
$ cargo build --release
```

## How to use
Currently, only twitter notifier is available.

1. Create your own app (<https://apps.twitter.com/>)
	* mcnotify does NOT provide consumer key.
1. Edit your configurations and save it to `~/.config/mcnotify/config.toml` (See `config.example.toml`).
1. Run.

## How to run
mcnotify is not daemon process. You can use nohup, tmux or systemd service to run mcnotify on background.

```console
$ ./target/release/mcnotify
```
