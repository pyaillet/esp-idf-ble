# esp-idf-ble

This project use as a workbench to enable BLE on the ESP32 microcontrollers family

## What's working ?

As of now, `esp-ble-example` is a rust port of [this esp-idf gatt_server example](https://github.com/espressif/esp-idf/tree/master/examples/bluetooth/bluedroid/ble/gatt_server).
It's using a custom Rust wrapper around the [esp-idf bluedroid BLE API](https://docs.espressif.com/projects/esp-idf/en/v4.4.2/esp32/api-reference/bluetooth/bt_le.html)

The goal is to complete the wrapper library and maybe make it usable elsewhere.

## How to use ?

Refer to [this repo](https://github.com/esp-rs/rust-build) to install the custom Rust ESP toolchain.
You should also install [cargo espflash](https://github.com/esp-rs/espflash) to ease the use of this project.

Then you can launch the following command to compile one of the example, flash it to your device and monitor the ESP32 serial:

`cargo espflash --example <example> --monitor --speed 921600 <device>`

## examples

- [ ] gatt_server
