powermeter
==========

The _powermeter_ crate contains a firmware written in Rust for the custom
designed power meter board based upon a ST Microelectronics STM32F042 MCU and a
Texas instruments INA260 power meter.

To build the firmware, simply call `cargo build --release` and find the
flashable binary in **target/thumbv6m-none-eabi/release/powermeter** .
Programming that binary to the board is left as an exercise for the reader for
now.

License
-------

[0-clause BSD license](LICENSE-0BSD.txt).
