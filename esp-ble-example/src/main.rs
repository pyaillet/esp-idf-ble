use std::ffi::CString;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use esp_idf_hal::delay;
use esp_idf_svc::netif::EspNetifStack;
use esp_idf_svc::nvs::EspDefaultNvs;
use esp_idf_svc::sysloop::EspSysLoopStack;
use esp_idf_sys::*;

use embedded_hal::blocking::delay::DelayUs;

use esp_idf_sys::c_types::c_void;
use ::log::*;

unsafe extern "C" fn bleprph_host_task(_param: *mut c_void) {
    nimble_port_run();

    nimble_port_freertos_deinit();
}

fn main() {
    esp_idf_sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    #[allow(unused)]
    let netif_stack = Arc::new(EspNetifStack::new().expect("Unable to init EspNetifStack"));
    #[allow(unused)]
    let sys_loop_stack = Arc::new(EspSysLoopStack::new().expect("Unable to init EspSysLoopStack"));

    #[allow(unused)]
    let default_nvs = Arc::new(EspDefaultNvs::new().unwrap());

    let mut delay = delay::Ets {};

    delay.delay_us(100_u32);

    info!("About to init NimBLE");
    esp!(unsafe { esp_nimble_hci_and_controller_init() }).expect("esp_nimble_hci_and_controller_init failed");

    unsafe { nimble_port_init() };

    unsafe { ble_svc_gap_init() };
    unsafe { ble_svc_gatt_init() };

    /*
    esp!(unsafe { ble_gatts_count_cfg(gatt_svr_svcs) });

    esp!(unsafe { ble_gatts_add_svcs(gatt_svr_svcs) });
    */

    let device_name = CString::new("nimble-bleprph").unwrap();

    esp!(unsafe { ble_svc_gap_device_name_set(device_name.as_ptr()) }).expect("Unable to set device name");

    unsafe { nimble_port_freertos_init(Some(bleprph_host_task)) };

    info!("NimBLE initialized");

    let params: ble_hs_adv_fields = ble_hs_adv_fields {
        _bitfield_1: 12,
    };

    loop {
        thread::sleep(Duration::from_secs(5));
    }
}
