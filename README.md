# floppy-driver-rs

**You can find the entire writeup for this project at** [https://floppy.cafe/](https://floppy.cafe)


This is a floppy driver written in baremetal rust. It sports a number of methods
capable of interfacing with the floppy drive and reading/writing from an IBM formatted
floppy disk. 

 - fdd.rs: the floppy disk driver
 - mfm.rs: the mfm encoding support functions
 - mfm.S: the lower level mfm encoding functions written in assembly
 - config.rs: pin configurations


This project is built off my own kernel, [teensycore](https://github.com/SharpCoder/teensycore).

## Installation

To properly build on a Linux machine, you'll need the following:

```bash
# Install build tools
sudo apt-get install build-essential gcc-arm-none-eabi jq

# Configure rust
rustup default nightly
rustup target add thumbv7em-none-eabihf
```

## Building

Run the build script and it will generate file`./out/floppy_driver_rs.hex` which can be flashed with the [teensy loader](https://www.pjrc.com/teensy/loader.html).

```bash
./build.sh
```

## License

[MIT](https://choosealicense.com/licenses/mit/)
