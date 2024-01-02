![CI](https://github.com/pyaillet/esp-idf-ble/workflows/Continuous%20integration/badge.svg)
![MIT/Apache-2.0 licensed](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)

⚠️ Support for BLE in Rust with ESP-IDF seems to now be provided by https://github.com/esp-rs/esp-idf-svc/blob/master/src/bt/ble, make sure to check it.

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

Targets:

- xtensa-esp32-espidf
- xtensa-esp32s2-espidf
- xtensa-esp32s3-espidf
- riscv32imc-esp-espidf

## Examples

- [ ] gatt_server
