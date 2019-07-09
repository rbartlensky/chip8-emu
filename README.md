# Chip8-emulator in Rust

This is a very simple project which implements a Chip8 emulator. There are some
bugs in the implementation, mainly due to the conflicting information found
online. Most games work, however, the gameplay feels very choppy. This might be
because I am not experienced with using
[Piston](https://crates.io/crates/piston_window), which I use to draw the state
of the emulator, and to take user input. Any help is welcome!

## Controls

| 1 | 2 | 3 | 4 |
|---|---|---|---|
| q | w | e | r |
| a | s | d | f |
| z | x | c | v |

## Playing a game

`cargo run --release -- --rom <path_to_rom>`
