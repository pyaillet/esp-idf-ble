use std::sync::mpsc::sync_channel;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use esp_idf_ble::advertise::AdvertiseData;
use esp_idf_ble::{
    AttributeValue, AutoResponse, BtUuid, EspBle, GattCharacteristic, GattDescriptor, GattService,
    GattServiceEvent,
};
use esp_idf_hal::delay;
// use esp_idf_hal::prelude::*;
use esp_idf_svc::netif::EspNetifStack;
use esp_idf_svc::nvs::EspDefaultNvs;
use esp_idf_svc::sysloop::EspSysLoopStack;
use esp_idf_sys::*;

use embedded_hal::blocking::delay::DelayUs;

use log::*;

fn main() {
    esp_idf_sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    #[allow(unused)]
    let netif_stack = Arc::new(EspNetifStack::new().expect("Unable to init Netif Stack"));
    #[allow(unused)]
    let sys_loop_stack = Arc::new(EspSysLoopStack::new().expect("Unable to init sys_loop"));

    #[allow(unused)]
    let default_nvs = Arc::new(EspDefaultNvs::new().unwrap());

    let mut delay = delay::Ets {};

    delay.delay_us(100_u32);

    let mut ble = EspBle::new("ESP32".into(), default_nvs).unwrap();

    let (s, r) = sync_channel(1);

    ble.register_gatt_service_application(1, move |gatts_if, reg| {
        if let GattServiceEvent::Register(reg) = reg {
            info!("Service registered with {:?}", reg);
            s.send(gatts_if).expect("Unable to send result");
        } else {
            warn!("What are you doing here??");
        }
    })
    .expect("Unable to register service");

    let svc_uuid = BtUuid::Uuid16(0x00FF);

    let svc = GattService::new_primary(svc_uuid, 4, 1);

    info!("GattService to be created: {:?}", svc);

    let gatts_if = r.recv().expect("Unable to receive value");

    let (s, r) = sync_channel(1);

    ble.create_service(gatts_if, svc, move |gatts_if, create| {
        if let GattServiceEvent::Create(create) = create {
            info!(
                "Service created with {{ \tgatts_if: {}\tstatus: {}\n\thandle: {}\n}}",
                gatts_if, create.status, create.service_handle
            );
            s.send(create.service_handle).expect("Unable to send value");
        }
    })
    .expect("Unable to create service");

    let svc_handle = r.recv().expect("Unable to receive value");

    ble.start_service(svc_handle, |_, start| {
        if let GattServiceEvent::StartComplete(start) = start {
            info!("Service started for handle: {}", start.service_handle);
        }
    })
    .expect("Unable to start ble service");

    let attr_value: AttributeValue<12> = AttributeValue::new_with_value(&[
        0x48, 0x65, 0x6C, 0x6C, 0x6F, 0x20, 0x57, 0x6F, 0x72, 0x6C, 0x64,
    ]);
    let charac = GattCharacteristic::new(
        BtUuid::Uuid16(0xff01),
        (ESP_GATT_PERM_READ | ESP_GATT_PERM_WRITE) as _,
        (ESP_GATT_CHAR_PROP_BIT_READ | ESP_GATT_CHAR_PROP_BIT_WRITE) as _,
        attr_value,
        AutoResponse::ByApp,
    );

    let (s, r) = sync_channel(1);

    ble.add_characteristic(svc_handle, charac, move |_, add_char| {
        if let GattServiceEvent::AddCharacteristicComplete(add_char) = add_char {
            info!("Attr added with handle: {}", add_char.attr_handle);
            s.send(add_char.attr_handle).expect("Unable to send value");
        }
    })
    .expect("Unable to add characteristic");

    let char_attr_handle = r.recv().expect("Unable to recv attr_handle");

    let data = ble
        .read_attribute_value(char_attr_handle)
        .expect("Unable to read characteristic value");
    info!("Characteristic values: {:?}", data);

    let cdesc = GattDescriptor::new(
        BtUuid::Uuid16(ESP_GATT_UUID_CHAR_CLIENT_CONFIG as u16),
        ESP_GATT_PERM_READ as _,
    );
    ble.add_descriptor(svc_handle, cdesc, |_, add_desc| {
        if let GattServiceEvent::AddDescriptorComplete(add_desc) = add_desc {
            info!("Descriptor added with handle: {}", add_desc.attr_handle);
        }
    })
    .expect("Unable to add characteristic");

    ble.register_read_handler(char_attr_handle, move |gatts_if, read| {
        let val = [0x48, 0x65, 0x6c, 0x6c, 0x6f];

        if let GattServiceEvent::Read(read) = read {
            esp_idf_ble::send(
                gatts_if,
                char_attr_handle,
                read.conn_id,
                read.trans_id,
                esp_gatt_status_t_ESP_GATT_OK,
                &val

            )
            .expect("Unable to send read response");
        }
    });

    ble.register_write_handler(char_attr_handle, move |gatts_if, write| {
        if let GattServiceEvent::Write(write) = write {
            if write.is_prep {
                warn!("Unsupported write");
            } else {
                let value = unsafe { std::slice::from_raw_parts(write.value, write.len as usize) };
                info!(
                    "Write event received for {} with: {:?}",
                    char_attr_handle, value
                );

                if write.need_rsp {
                    esp_idf_ble::send(
                        gatts_if,
                        char_attr_handle,
                        write.conn_id,
                        write.trans_id,
                        esp_gatt_status_t_ESP_GATT_OK,
                        &[]
                                            )
                    .expect("Unable to send response");
                }
            }
        }
    });

    let adv_data = AdvertiseData {
        include_name: true,
        include_txpower: false,
        min_interval: 6,
        max_interval: 16,
        service_uuid: Some(BtUuid::Uuid128([
            0xfb, 0x34, 0x9b, 0x5f, 0x80, 0x00, 0x00, 0x80, 0x00, 0x10, 0x00, 0x00, 0xFF, 0x00,
            0x00, 0x00,
        ])),
        flag: (ESP_BLE_ADV_FLAG_GEN_DISC | ESP_BLE_ADV_FLAG_BREDR_NOT_SPT) as _,
        ..Default::default()
    };
    ble.configure_advertising_data(adv_data, |_| {
        info!("advertising configured");
    })
    .expect("Failed to configure advertising data");

    let scan_rsp_data = AdvertiseData {
        include_name: false,
        include_txpower: true,
        set_scan_rsp: true,
        service_uuid: Some(BtUuid::Uuid128([
            0xfb, 0x34, 0x9b, 0x5f, 0x80, 0x00, 0x00, 0x80, 0x00, 0x10, 0x00, 0x00, 0xFF, 0x00,
            0x00, 0x00,
        ])),
        ..Default::default()
    };

    ble.configure_advertising_data(scan_rsp_data, |_| {
        info!("Advertising configured");
    })
    .expect("Failed to configure advertising data");

    ble.start_advertise(|_| {
        info!("advertising started");
    })
    .expect("Failed to start advertising");

    loop {
        thread::sleep(Duration::from_secs(5));
    }
}
