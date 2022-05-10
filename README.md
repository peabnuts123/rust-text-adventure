# Rust Text Adventure

This is a simple project that interacts with the [Our Text Adventure](https://github.com/peabnuts123/our-text-adventure) API and lets your play from the command line.

It was mostly created so that I could learn about programming in Rust.

## How to play

See the [Our Text Adventure](https://text-adventure.winsauce.com) site for more information, or type `/help` to see all the available commands.

This project cannot currently be used to create new data for the game, only to interact with the game as a player.

```
> /help
List of commands:
/inventory
    List your inventory

/screen-id
(alias: /screen)
    Print the current screen's
    id (useful when creating a
    new screen)

/look
(alias: /whereami)
(alias: /where)
(alias: /repeat)
(alias: /again)
    Print the current screen
    again

/help
(alias: /?)
    Print this help message

/exit
(alias: /quit)
    Quit the game
```