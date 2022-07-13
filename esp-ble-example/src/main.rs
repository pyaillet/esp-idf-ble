use std::sync::Arc;
use std::thread;
use std::time::Duration;

use esp_ble::advertise::{AdvertiseData, AppearanceCategory};
use esp_ble::{
    AttributeValue, AutoResponse, BtUuid, EspBle, GattApplication, GattCharacteristic, GattCharacteristicDesc, GattService,
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

    let mut ble = EspBle::new("Test device".into(), default_nvs).unwrap();
    let application = Arc::new(Mutex::new(GattApplication::new(1)));
    block_on(async {
        let _ = ble
            .register_gatt_service_application(application.clone())
            .await;
        info!("application registered");

        let svc = GattService::new(BtUuid::Uuid16(0x180A), 4, 1);
        let svc_handle = ble
            .create_service(application, svc)
            .await
            .expect("Unable to create service");
        let _ = ble.start_service(svc_handle).await;

        let attr_value: AttributeValue<10> = AttributeValue::new();
        let charac = GattCharacteristic::new(
            BtUuid::Uuid16(0x2A29),
            ESP_GATT_PERM_READ as _,
            ESP_GATT_CHAR_PROP_BIT_READ as _,
            attr_value,
            AutoResponse::ByGatt,
        );
        let _ = ble.add_characteristic(svc_handle, charac).await;

        let cdesc = GattCharacteristicDesc::new(BtUuid::Uuid16(0x2A29), ESP_GATT_PERM_READ as _);
        let _ = ble.add_characteristic_desc(svc_handle, cdesc).await;

        let adv_data = AdvertiseData {
            manufacturer: Some("Espressif".into()),
            include_name: true,
            appearance: AppearanceCategory::Watch,
            ..Default::default()
        };
        let _ = ble.configure_advertising_data(adv_data).await;

        let scan_rsp_data = AdvertiseData {
            set_scan_rsp: true,
            include_name: true,
            manufacturer: Some("Espressif".into()),
            appearance: AppearanceCategory::Watch,
            ..Default::default()
        };
        let _ = ble.configure_advertising_data(scan_rsp_data).await;

        info!("advertising configured");
        let _ = ble.start_advertise().await;
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
