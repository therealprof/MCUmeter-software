# powermeter

The _powermeter_ crate contains a firmware written in Rust for the custom
designed power meter board based upon a ST Microelectronics STM32F042 MCU and a
Texas instruments INA260 power meter.

## Building

To build the firmware, simply call `cargo build --release` and find the
flashable binary in **target/thumbv6m-none-eabi/release/powermeter** .

## Flashing

### OpenOCD

To use OpenOCD you will need to have OpenOCD installed and a separate SWD debugger available. The way how to use OpenOCD unfortunately depends on the debugging interface used. One of the easier way and OSS ways to flash is by getting a DAPLink interface (e.g. by buying a NXP FRDM board and populating the SWD header) and using the included script.

```
# openocd_program.sh target/thumbv6m-none-eabi/release/powermeter
```

### Via USB / dfu-util

To use this method you will to have cargo-binutils installed, this can be achieved by calling `cargo install binutils` **outside** of the software directory.

You will also need to have `dfu-util` installed.

Once those prerequisites are fulfilled you will need to convert the generated ELF binary into a raw binary file:
```
# cargo objcopy -- -O binary target/thumbv6m-none-eabi/release/powermeter powermeter.bin
```

To flash this file to the device you will need turn it into bootloader mode by shorting out the pins labelled **BOOT** and plugging the power. If the device is in **DFU** mode, `dfu-util` will tell you:

```
# dfu-util -l
dfu-util 0.9

Copyright 2005-2009 Weston Schmidt, Harald Welte and OpenMoko Inc.
Copyright 2010-2016 Tormod Volden and Stefan Schmidt
This program is Free Software and has ABSOLUTELY NO WARRANTY
Please report bugs to http://sourceforge.net/p/dfu-util/tickets/

Found DFU: [0483:df11] ver=2200, devnum=50, cfg=1, intf=0, path="20-11.2", alt=1, name="@Option Bytes  /0x1FFFF800/01*016 e", serial="FFFFFFFEFFFF"
Found DFU: [0483:df11] ver=2200, devnum=50, cfg=1, intf=0, path="20-11.2", alt=0, name="@Internal Flash  /0x08000000/032*0001Kg", serial="FFFFFFFEFFFF"
```

Once you've reached this stage you can flash the *powermeter.bin* file using:

```
# dfu-util -a0 --dfuse-address 0x08000000 -D powermeter.bin
dfu-util 0.9

Copyright 2005-2009 Weston Schmidt, Harald Welte and OpenMoko Inc.
Copyright 2010-2016 Tormod Volden and Stefan Schmidt
This program is Free Software and has ABSOLUTELY NO WARRANTY
Please report bugs to http://sourceforge.net/p/dfu-util/tickets/

dfu-util: Invalid DFU suffix signature
dfu-util: A valid DFU suffix will be required in a future dfu-util release!!!
Opening DFU capable USB device...
ID 0483:df11
Run-time device DFU version 011a
Claiming USB DFU Interface...
Setting Alternate Setting #0 ...
Determining device status: state = dfuIDLE, status = 0
dfuIDLE, continuing
DFU mode device DFU version 011a
Device returned transfer size 2048
DfuSe interface name: "Internal Flash  "
Downloading to address = 0x08000000, size = 13048
Download	[=========================] 100%        13048 bytes
Download done.
File downloaded successfully
```

License
-------

[0-clause BSD license](LICENSE-0BSD.txt).
