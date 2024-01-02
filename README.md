![CI](https://github.com/pyaillet/esp-idf-ble/workflows/Continuous%20integration/badge.svg)
![MIT/Apache-2.0 licensed](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)

# esp-idf-ble

This project aims at providing a safe Rust wrapper of `esp-idf` to enable BLE on the ESP32 microcontrollers family

## What's working ?

It's using a custom Rust wrapper around the [esp-idf bluedroid BLE API](https://docs.espressif.com/projects/esp-idf/en/v4.4.2/esp32/api-reference/bluetooth/bt_le.html)
As of now, only the `gatt_server` example is partially implemented. IT is a rust port of [this esp-idf gatt_server example](https://github.com/espressif/esp-idf/tree/master/examples/bluetooth/bluedroid/ble/gatt_server).

The goal is to complete the wrapper library and maybe make it usable elsewhere.

## How to use ?

Refer to [this repo](https://github.com/esp-rs/rust-build) to install the custom Rust ESP toolchain.
You should also install [cargo espflash](https://github.com/esp-rs/espflash) to ease the use of this project.

Then you can launch the following command to compile one of the example, flash it to your device and monitor the ESP32 serial:

`cargo espflash --example <example> --monitor --speed 921600 <device> --target <target>`

If you want to try the examples, set the target according to your device:
- ESP32: xtensa-esp32-espidf
- ESP32-S2: xtensa-esp32s2-espidf
- ESP32-S3: xtensa-esp32s3-espidf
- ESP32-C3: riscv32imc-esp-espidf

If you want to use it as a library, set the target in your own projet in the `.cargo/config.toml` file (you can check the file ()[./.cargo/config.toml] for reference)

## Examples

- [ ] gatt_server
