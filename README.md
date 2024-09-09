
# Multi-platform XO-CHIP Emulator

![Build action](https://github.com/dustinbowers/rust-chip8/actions/workflows/rust.yml/badge.svg)
![Build action](https://github.com/dustinbowers/rust-chip8/actions/workflows/wasm.yml/badge.svg)

An XO-CHIP emulator written in [Rust](https://www.rust-lang.org/) using [Macroquad](https://macroquad.rs/) for rendering, meant to be compiled to WASM. Includes support for CHIP-8 predecessors as well

Currently supported extensions:
- [XO-CHIP](https://johnearnest.github.io/Octo/docs/XO-ChipSpecification.html) (Extended to support 4-bit planes / 16 colors!)
- [CHIP-8](https://en.wikipedia.org/wiki/CHIP-8)
- [Super-Chip 1.1 (Modern)](https://github.com/Chromatophore/HP48-Superchip)
- [Super-Chip 1.1 (Legacy)](https://github.com/Chromatophore/HP48-Superchip/blob/master/binaries/SCHIP_origin.txt)
  - with caveats

## Live Demo

Play with the live demo here: https://dustinbowers.com/rust-chip8

## Screenshots

<img src="https://github.com/dustinbowers/rust-chip8/blob/main/screenshots/super-neat-boy.gif" width="40%"> <img src="https://github.com/dustinbowers/rust-chip8/blob/main/screenshots/nyancat.gif" width="40%">
<img src="https://github.com/dustinbowers/rust-chip8/blob/main/screenshots/alien-inv8sion.gif" width="40%"> <img src="https://github.com/dustinbowers/rust-chip8/blob/main/screenshots/t8nks.gif" width="40%">



## What is CHIP-8?

> Chip-8 is a simple, interpreted, programming language which was first used on some do-it-yourself computer systems in the late 1970s and early 1980s. The COSMAC VIP, DREAM 6800, and ETI 660 computers are a few examples. These computers typically were designed to use a television as a display, had between 1 and 4K of RAM, and used a 16-key hexadecimal keypad for input. The interpreter took up only 512 bytes of memory, and programs, which were entered into the computer in hexadecimal, were even smaller.

[(source)](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM#1.0)

## Build

The Makefile includes various targets:

| Target                     | Description                                              |
|-----------------------------|----------------------------------------------------------|
| `make build`                | Build debug binary                                       |
| `make release`              | Build release binary                                     |
| `make wasm`                 | Build debug WASM                                         |
| `make wasm-release`         | Build release WASM                                       |
| `make web-server`           | Run `basic-http-server` from the `./dist` directory at http://localhost:4000 |
| `make build-test-web`       | Build debug WASM and run webserver from `./dist`         |
| `make build-test-web-release`| Build release WASM and run webserver from `./dist`      |


## Usage

Binary:
```
Usage: chip8 <Filename> <CHIP Mode> <Ticks-per-frame>

<Filename> - path to ROM File
<CHIP Mode>
        1 - CHIP-8
        2 - SuperChip Modern
        3 - SuperChip Legacy
        4 - XO-Chip
<Ticks-per-frame> - Number of instructions emulated per frame
```

Locally hosted WASM:
```
make build-test-web-release
```
and browse to http://localhost:4000


## Note

All ROMs in this repo were gathered together from various places around the internet, and credit for each goes to their respective authors

## Useful links 

- https://en.wikipedia.org/wiki/CHIP-8
- Opcodes - https://chip8.gulrak.net/
- Chip-8 Rom Archive - https://johnearnest.github.io/chip8Archive/?sort=platform
