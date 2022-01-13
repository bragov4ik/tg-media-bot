# Telegram media bot

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

### Launching
You can use `cargo install` or manually clone into repository then either install on a machine or build and run in the folder.

* Start redis server
* Specify bot API token in `TELOXIDE_TOKEN` enviromental variable

#### Using cargo install
* Type `cargo install --git https://github.com/bragov4ik/tg-media-bot.git`
* Run the bot using `tg-media-bot` command *(if cargo installation folder is in your `PATH`)*
#### Manual installation
* Clone the repository into any folder
* Use `cargo install` inside the folder
* Write `tg-media-bot` to run *(if does not work check that cargo installation location is in your `PATH`)*
#### Manual portable launch
* Clone the repository into any folder
* Type `cargo run --release` to build and run the project

### Bugs/problems
If any bugs related to the code were found, create an issue with its description.

### Planned work/features
Kind of sorted according to importance (higher - more preferable)
* more automatic deployment
* add support for any media
* inline search
* more elegant way to handle common commands
* marking symbol specification (colons may cause conflict) *however can be avoided right now by not giving admin rights to the bot, so it does not see all the messages*
* resolve TODOs *(not critical, just better practices)*
* add proper *(unit)* tests
