use std::sync::Arc;
use std::thread;
use std::time::Duration;

use esp_ble::advertise::{AdvertiseData, AdvertiseType, AppearanceCategory, RawAdvertiseData};
use esp_ble::{
    AttributeValue, AutoResponse, BtUuid, EspBle, GattApplication, GattCharacteristic,
    GattCharacteristicDesc, GattService,
};
use esp_idf_hal::delay;
use esp_idf_hal::mutex::Mutex;
// use esp_idf_hal::prelude::*;
use esp_idf_svc::netif::EspNetifStack;
use esp_idf_svc::nvs::EspDefaultNvs;
use esp_idf_svc::sysloop::EspSysLoopStack;
use esp_idf_sys::*;

use embedded_hal::blocking::delay::DelayUs;
// use embedded_hal::digital::v2::OutputPin;

use anyhow::Result;

use log::*;
use smol::future::block_on;

fn main() {
    init_esp().expect("Error initializing ESP");

    #[allow(unused)]
    let default_nvs = Arc::new(EspDefaultNvs::new().unwrap());

    let mut delay = delay::Ets {};

    delay.delay_us(100_u32);

    let mut ble = EspBle::new("ESP32".into(), default_nvs).unwrap();
    let application = Arc::new(Mutex::new(GattApplication::new(1)));
    block_on(async {
        let _ = ble
            .register_gatt_service_application(application.clone())
            .await;
        info!("application registered");

        let svc_uuid = BtUuid::Uuid16(0x00ff);

        let svc = GattService::new(svc_uuid.clone(), 4, 1);
        let svc_handle = ble
            .create_service(application, svc)
            .await
            .expect("Unable to create service");
        ble.start_service(svc_handle)
            .await
            .expect("Unable to start ble service");

        let attr_value: AttributeValue<10> = AttributeValue::new();
        let charac = GattCharacteristic::new(
            BtUuid::Uuid16(0xff01),
            ESP_GATT_PERM_READ as _,
            ESP_GATT_CHAR_PROP_BIT_READ as _,
            attr_value,
            AutoResponse::ByGatt,
        );
        ble.add_characteristic(svc_handle, charac)
            .await
            .expect("Unable to add characteristic");

        let cdesc = GattCharacteristicDesc::new(BtUuid::Uuid16(0x3333), ESP_GATT_PERM_READ as _);
        ble.add_characteristic_desc(svc_handle, cdesc)
            .await
            .expect("Unable to add characteristic");

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
        ble.configure_advertising_data(adv_data)
            .await
            .expect("Failed to configure advertising data");

        info!("advertising configured");

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

        ble.configure_advertising_data(scan_rsp_data)
            .await
            .expect("Failed to configure advertising data");

        ble.start_advertise()
            .await
            .expect("Failed to start advertising");
        info!("advertising started");
    });

    loop {
        thread::sleep(Duration::from_secs(5));
    }
}

fn init_esp() -> Result<()> {
    esp_idf_sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    #[allow(unused)]
    let netif_stack = Arc::new(EspNetifStack::new()?);
    #[allow(unused)]
    let sys_loop_stack = Arc::new(EspSysLoopStack::new()?);

    Ok(())
}
