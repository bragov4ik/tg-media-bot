# Telegram media bot
![Clippy workflow](https://github.com/bragov4ik/tg-media-bot/actions/workflows/clippy_check.yml/badge.svg)
![Cargo workflow](https://github.com/bragov4ik/tg-media-bot/actions/workflows/cargo_check.yml/badge.svg)
![Formatting workflow](https://github.com/bragov4ik/tg-media-bot/actions/workflows/formatting.yml/badge.svg)

Telegram bot written in rust for aliasing different media (currently only stickers are supported). 

## How it works

Initially, no aliases are specified. They can be added using `/add` command, which starts an addition process:

![add_demo](https://user-images.githubusercontent.com/8144358/149161070-f11f947b-44a2-4c2e-b48f-ab291ba818e5.gif)

After that, you can use specified aliases enclosed in colons in your messages. The bot will send the associated media to the chat:

![replacing_demo](https://user-images.githubusercontent.com/8144358/149163920-cac6a7cc-8379-4b55-a172-b6a78270edac.gif)

## Usage

The bot is (hopefully still) running at http://t.me/textmedia_bot. 
* Add it to a chat (or start a conversation in PM)
* *(If using in chat)* Give admin rights if you wish all messages in the chat to be seen.
* Use it according to `/start` and `/help`

## How to run it by yourself

### Requirements
* Rust/Cargo 1.56+
* Redis 6.2+

Older versions may work, however they were not tested.

### Usage
You can download precompiled binaries from releases page or build the project by yourself.

Before launching the bot make sure to
* Start redis server
* Specify bot API token in `TELOXIDE_TOKEN` enviromental variable

#### Using precompiled binary
* Download binary for your platform from [releases page](https://github.com/bragov4ik/tg-media-bot/releases)
* Unpack it
* Launch from command line

#### Manual building
* Type `cargo install --git https://github.com/bragov4ik/tg-media-bot.git`
* Run the bot using `tg-media-bot` command *(if cargo installation folder is in your `PATH`)*

Or compile using any other method like `cargo run --release`

#### Arguments
There is one optional argument - Redis address. You should specify only the address itself, without `redis://` prefix.

Usage example: `tg-media-bot 127.0.0.1`

### Bugs/problems
If any bugs related to the code were found, create an issue with its description.

### Planned work/features
Kind of sorted according to importance (higher - more preferable)
* add proper *(unit)* tests
* add support for any media
* inline search
* more elegant way to handle common commands
* marking symbol specification (colons may cause conflict) *however can be avoided right now by not giving admin rights to the bot, so it does not see all the messages*
* resolve TODOs *(not critical, just better practices)*
