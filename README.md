
# Multi-platform Chip-8 Emulator

![Build action](https://github.com/dustinbowers/rust-chip8/actions/workflows/rust.yml/badge.svg)

An emulator written in [Rust](https://www.rust-lang.org/) using [Macroquad](https://macroquad.rs/) for rendering

Currently supported platforms:
- [XO-CHIP](https://johnearnest.github.io/Octo/docs/XO-ChipSpecification.html)
- [CHIP-8](https://en.wikipedia.org/wiki/CHIP-8)
- [Super-Chip 1.1 (Modern)](https://github.com/Chromatophore/HP48-Superchip)
- [Super-Chip 1.1 (Legacy)](https://github.com/Chromatophore/HP48-Superchip/blob/master/binaries/SCHIP_origin.txt)

## What is CHIP-8?

> Chip-8 is a simple, interpreted, programming language which was first used on some do-it-yourself computer systems in the late 1970s and early 1980s. The COSMAC VIP, DREAM 6800, and ETI 660 computers are a few examples. These computers typically were designed to use a television as a display, had between 1 and 4K of RAM, and used a 16-key hexadecimal keypad for input. The interpreter took up only 512 bytes of memory, and programs, which were entered into the computer in hexadecimal, were even smaller.

[(source)](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM#1.0)

## Useful links 

- https://en.wikipedia.org/wiki/CHIP-8
- Opcodes - https://chip8.gulrak.net/
- Chip-8 Rom Archive - https://johnearnest.github.io/chip8Archive/?sort=platform
