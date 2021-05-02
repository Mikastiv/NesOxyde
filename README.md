# NesOxyde

A NES emulator 100% written in Rust

## Overview

This project is my first big programming project and also my first emulator (I'm not counting my Chip8 emulator because it was very simple compared to this one).

I chose Rust because it's fast and it's by far my favorite language. Also all the code is 100% safe Rust!

This was my 4th attempt at coding a NES emulator and I finally succeded! The emulator is not cycle accurate, but all the games I've tried work pretty well.

## Usage

The program is needs libsdl2 to run and libsdl2-devel to compile.
It works on Linux, Windows and MacOS

Launch: ./nesoxyde [SyncMode] \<iNES File\>

SyncMode: -A (default) or -V

- Audio sync (-A). The emulation is synced with the audio sample rate (44100Hz). Can cause frame lag.
- Video sync (-V). The emulation is synced with the video refresh rate of 60fps. Can cause audio pops and cracks.

## Controls

R -> Reset  
Esc -> Close emulator  
1 -> Volume down  
2 -> Volume up

Key mappings can be changed in main.rs  
Defaults:

- A -> B
- S -> A
- Z -> Select
- X -> Start
- UpArrow ->  Up
- DownArrow -> Down
- LeftArrow -> Left
- RightArrow -> Right

## TODOs

- Add a save/load state system

## Screenshots

![Super Mario Bros](/screenshots/smb.png "Super Mario Bros")
![Super Mario Bros 3](/screenshots/smb3.png "Super Mario Bros 3")
![Zelda](/screenshots/zelda.png "Zelda")
![Castlevania](/screenshots/castlevania.png "Castlevania")
