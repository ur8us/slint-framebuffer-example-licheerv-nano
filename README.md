
# Slint framebuffer example running on LicheeRV Nano development board 

Thiis is a demo of compiling and running a Slint UI program on a Risc-V 64-bit microprocessor under Linux, using Linux framebuffer, **with double buffering**.

Pressing the USER button (/dev/input/event0) increments the value.

Touch screen (/dev/input/event1) is also supported.

## Hardware

LicheeRV Nano - https://wiki.sipeed.com/hardware/en/lichee/RV_Nano/1_intro.html

IPS 2.28 inch 31-pin ST7701S-based display, resolution is 368*552

(For the large ILI9881C-based display, use sample-dsi to set the DSI mode.)

## Prepare the target

Flash the SD card image: https://github.com/sipeed/LicheeRV-Nano-Build/releases/tag/20250804

Enable framebuffer and configure the 2.28 inch LCD according to the **LCD** section of the instructions: https://wiki.sipeed.com/hardware/en/lichee/RV_Nano/5_peripheral.html

## Prepare the host environment

To link the program for the riscv64gc-unknown-linux-musl architecture, install riscv64-linux-musl-gcc from https://musl.cc/ . You will need the riscv64-linux-musl-cross.tgz file.

## Compile and run the program

cargo run --release

The run-on-target.sh script will be started, uploading the executable file to the target board and running it. Make sure that this sh file contains the actual IP address of the board.

![](Screenshot-02.png)


Small LCD, 368*552:

![](photo-lcd.jpeg)


Large display, 720*1280

![](photo-lcd-large.jpeg)


# Old README.md from https://github.com/nilclass/slint-framebuffer-example is below

https://github.com/nilclass/slint-framebuffer-example/blob/main/README.md

