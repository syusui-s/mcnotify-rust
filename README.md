mcnotify-rust: Minecraft status notifier written in Rust
===

[![LICENSE](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![BuildStatus](https://github.com/syusui-s/mcnotify-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/syusui-s/mcnotify-rust/actions/workflows/ci.yml)


## Overview

This app sends notifications when someone joins / leaves your Minecraft server (Java version).

Supported notifications:

* Server stopped / recovered
* Player join / left
* Current players and the number of players

Supported notifiers:

* Command
* Twitter
* [IFTTT Maker Channel (Webhook)](https://ifttt.com/maker_webhooks)
* Stdout (in order to see what text will be notified)

## Download

Please refer [Releases](https://github.com/syusui-s/mcnotify-rust/releases) to download binaries.

Currently, **ONLY** Linux x86_64 binary is available on Releases.

If you need a binary for another platforms such as macOS, Windows and RaspberryPi (armhf Linux), please build it by yourself.

## How to build

```console
$ cargo build --release
```

## How to create the configuration file

The default path is `~/.config/mcnotify/config.toml`.

Please take a look at `config.example.toml`.

## How to run

### Normal execution

You can run the binary after the build.

```console
$ ./target/release/mcnotify
```

If you want to see a help message, use `--help` options.

```console
$ mcnotify --help
```

## Run in background

mcnotify is NOT daemon process.

You can use nohup, tmux or systemd service to run mcnotify in background.

Systemd service example:

```systemd
[Unit]
Description=Minecraft Notifier

[Service]
WorkingDirectory=/
User=minecraft
ExecStart=/usr/local/bin/mcnotify -c /home/minecraft/mcnotify.server_a.toml
Restart=always
RestartSec=30
Environment=RUST_BACKTRACE=1

[Install]
WantedBy=multi-user.target
```
