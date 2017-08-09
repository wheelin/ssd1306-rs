# ssd1306-rs
This repository contains code to communicate with i2c ssd1306 controlled oled displays with the [Rust](https://www.rust-lang.org/en-US/) language. It works on top of [i2cdev](https://github.com/rust-embedded/rust-i2cdev) and is intended to work on the raspberry pi. You must download
the raspberry pi toolchain and insert this :
```
[target.arm-unknown-linux-gnueabihf]
linker = "<path-to-raspberry-pi-toolchain>/tools/arm-bcm2708/arm-rpi-4.9.3-linux-gnueabihf/bin/arm-linux-gnueabihf-gcc"
```
to your .cargo/config file. You also must switch to nighly rust with following command (rustup):
```
rustup default nightly
```
