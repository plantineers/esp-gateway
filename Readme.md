# ESP-Gateway

This is a simple gateway to forward data from ESP-Now to http for the Project PlantBuddy. It runs on an ESP32C3.

## Setup

1. Install espflash
2. Switch to Rust nightly using `rustup override set nightly`
3. Run `cargo run --release` to build and flash the firmware

## Adapting for your own use case

If you want to reuse this code for your own use case, you need to change the following things:

1. Change the data structures in `src/main.rs` to match your own data
2. Encode the data sent by your other ESP using the rust postcard library
3. Adapt serial-forwarder to your own use case

## Example Output

![Example Output](example.png)

## Structure

The gateway is intentionally split into two parts, one being the ESP acting as a receiver that also decodes the messages(since Linux does not have ESP-Now support and patching the Linux kernel to do so via another project that might not work anymore with newer kernels was really not in scope). And also because it would be hard currently to do HTTPS on the ESP32C3 using esp-hal. 

The other part is the actual gateway that runs on Linux and receives the data from the ESP via serial, does a small transformation and forwards it to the backend via http.