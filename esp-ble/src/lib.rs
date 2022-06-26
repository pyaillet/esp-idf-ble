use std::{
    collections::HashMap,
    ffi::CString,
    sync::{atomic::AtomicBool, Arc, Mutex},
};

use anyhow::Result;
use esp_idf_sys::{esp, EspError};

pub mod gap;
pub mod gatt;

pub use gap::*;
pub use gatt::*;

const PROFILE_A_APP_ID: u16 = 0;

const GATTS_SERVICE_UUID_TEST_A: u16 = 0x00FF;
const GATTS_CHAR_UUID_TEST_A: u16 = 0xFF01;
const GATTS_DESCR_UUID_TEST_A: u16 = 0x3333;
const GATTS_NUM_HANDLE_TEST_A: u16 = 4;

const ADV_SERVICE_UUID128: &[u8; 32] = &[
    /* LSB <--------------------------------------------------------------------------------> MSB */
    //first uuid, 16bit, [12],[13] is the value
    0xfb, 0x34, 0x9b, 0x5f, 0x80, 0x00, 0x00, 0x80, 0x00, 0x10, 0x00, 0x00, 0xEE, 0x00, 0x00, 0x00,
    //second uuid, 32bit, [12], [13], [14], [15] is the value
    0xfb, 0x34, 0x9b, 0x5f, 0x80, 0x00, 0x00, 0x80, 0x00, 0x10, 0x00, 0x00, 0xFF, 0x00, 0x00, 0x00,
];

const SERVICE_UUID: &[u8; 16] = &[
    0x5a, 0xc9, 0xbc, 0x5e, 0xf8, 0xba, 0x48, 0xd4, 0x89, 0x08, 0x98, 0xb8, 0x0b, 0x56, 0x6e, 0x49,
];
const COMMAND_UUID: &[u8; 16] = &[
    0xbc, 0xca, 0x87, 0x2f, 0x1a, 0x3e, 0x44, 0x91, 0xb8, 0xec, 0xbf, 0xc9, 0x3c, 0x5d, 0xd9, 0x1a,
];

const ADV_PARAMS: esp_idf_sys::esp_ble_adv_params_t = esp_idf_sys::esp_ble_adv_params_t {
    adv_int_min: 0x20,
    adv_int_max: 0x40,
    adv_type: esp_idf_sys::esp_ble_adv_type_t_ADV_TYPE_IND,
    own_addr_type: esp_idf_sys::esp_ble_addr_type_t_BLE_ADDR_TYPE_PUBLIC,
    peer_addr: [0; 6],
    peer_addr_type: 0,
    channel_map: esp_idf_sys::esp_ble_adv_channel_t_ADV_CHNL_ALL,
    adv_filter_policy: esp_idf_sys::esp_ble_adv_filter_t_ADV_FILTER_ALLOW_SCAN_ANY_CON_ANY,
};

fn bt_controller_default_config() -> esp_idf_sys::esp_bt_controller_config_t {
    esp_idf_sys::esp_bt_controller_config_t {
        controller_task_stack_size: esp_idf_sys::ESP_TASK_BT_CONTROLLER_STACK as u16,
        controller_task_prio: esp_idf_sys::ESP_TASK_BT_CONTROLLER_PRIO as u8,
        hci_uart_no: esp_idf_sys::BT_HCI_UART_NO_DEFAULT as u8,
        hci_uart_baudrate: esp_idf_sys::BT_HCI_UART_BAUDRATE_DEFAULT,
        scan_duplicate_mode: esp_idf_sys::SCAN_DUPLICATE_MODE as u8,
        scan_duplicate_type: esp_idf_sys::SCAN_DUPLICATE_TYPE_VALUE as u8,
        normal_adv_size: esp_idf_sys::NORMAL_SCAN_DUPLICATE_CACHE_SIZE as u16,
        mesh_adv_size: esp_idf_sys::MESH_DUPLICATE_SCAN_CACHE_SIZE as u16,
        send_adv_reserved_size: esp_idf_sys::SCAN_SEND_ADV_RESERVED_SIZE as u16,
        controller_debug_flag: esp_idf_sys::CONTROLLER_ADV_LOST_DEBUG_BIT,
        mode: esp_idf_sys::esp_bt_mode_t_ESP_BT_MODE_BLE as u8,
        ble_max_conn: esp_idf_sys::CONFIG_BTDM_CTRL_BLE_MAX_CONN_EFF as u8,
        bt_max_acl_conn: esp_idf_sys::CONFIG_BTDM_CTRL_BR_EDR_MAX_ACL_CONN_EFF as u8,
        bt_sco_datapath: esp_idf_sys::CONFIG_BTDM_CTRL_BR_EDR_SCO_DATA_PATH_EFF as u8,
        auto_latency: esp_idf_sys::BTDM_CTRL_AUTO_LATENCY_EFF != 0,
        bt_legacy_auth_vs_evt: esp_idf_sys::BTDM_CTRL_LEGACY_AUTH_VENDOR_EVT_EFF != 0,
        bt_max_sync_conn: esp_idf_sys::CONFIG_BTDM_CTRL_BR_EDR_MAX_SYNC_CONN_EFF as u8,
        ble_sca: esp_idf_sys::CONFIG_BTDM_BLE_SLEEP_CLOCK_ACCURACY_INDEX_EFF as u8,
        pcm_role: esp_idf_sys::CONFIG_BTDM_CTRL_PCM_ROLE_EFF as u8,
        pcm_polar: esp_idf_sys::CONFIG_BTDM_CTRL_PCM_POLAR_EFF as u8,
        hli: esp_idf_sys::BTDM_CTRL_HLI != 0,
        magic: esp_idf_sys::ESP_BT_CONTROLLER_CONFIG_MAGIC_VAL,
    }
}

struct BleServer {
    applications: HashMap<esp_idf_sys::esp_gatt_if_t, BleApp>,
}

static ADVERTISE_CONFIGURED: AtomicBool = AtomicBool::new(false);
static SCAN_RESPONSE_CONFIGURED: AtomicBool = AtomicBool::new(false);

impl Default for BleServer {
    fn default() -> Self {
        BleServer {
            applications: HashMap::new(),
        }
    }
}

#[derive(Default)]
struct BleApp {
    gatts_if: esp_idf_sys::esp_gatt_if_t,
    service_id: Option<esp_idf_sys::esp_gatt_srvc_id_t>,
    service_handle: Option<u16>,
    char_uuid: Option<esp_idf_sys::esp_bt_uuid_t>,
    char_handle: Option<u16>,
    descr_uuid: Option<esp_idf_sys::esp_bt_uuid_t>,
    descr_handle: Option<u16>,
    conn_id: Option<u16>,
    a_property: Option<u8>,
}

lazy_static::lazy_static! {
    static ref DEVICE_NAME: CString = CString::new("Test device").unwrap();
    static ref MANUFACTURER_NAME: CString = CString::new("Home").unwrap();
}

impl BleApp {
    fn get_adv_data() -> esp_idf_sys::esp_ble_adv_data_t {
        esp_idf_sys::esp_ble_adv_data_t {
            set_scan_rsp: false,
            include_name: true,
            include_txpower: false,
            min_interval: 0x0006, //slave connection min interval, Time = min_interval * 1.25 msec
            max_interval: 0x0010, //slave connection max interval, Time = max_interval * 1.25 msec
            appearance: 0x00,
            manufacturer_len: 0, //TEST_MANUFACTURER_DATA_LEN,
            p_manufacturer_data: std::ptr::null_mut(), //&test_manufacturer[0],
            service_data_len: 0,
            p_service_data: std::ptr::null_mut(),
            service_uuid_len: ADV_SERVICE_UUID128.len() as u16,
            p_service_uuid: ADV_SERVICE_UUID128.clone().as_mut_ptr(),
            flag: (esp_idf_sys::ESP_BLE_ADV_FLAG_GEN_DISC
                | esp_idf_sys::ESP_BLE_ADV_FLAG_BREDR_NOT_SPT) as u8,
        }
    }

    fn get_scan_rsp_data() -> esp_idf_sys::esp_ble_adv_data_t {
        esp_idf_sys::esp_ble_adv_data_t {
            set_scan_rsp: true,
            include_name: true,
            include_txpower: true,
            min_interval: 0x0000,
            max_interval: 0x0000,
            appearance: 0x00,
            manufacturer_len: 0, //TEST_MANUFACTURER_DATA_LEN,
            p_manufacturer_data: std::ptr::null_mut(), //&test_manufacturer[0],
            service_data_len: 0,
            p_service_data: std::ptr::null_mut(),
            service_uuid_len: ADV_SERVICE_UUID128.len() as u16,
            p_service_uuid: ADV_SERVICE_UUID128.clone().as_mut_ptr(),
            flag: (esp_idf_sys::ESP_BLE_ADV_FLAG_GEN_DISC
                | esp_idf_sys::ESP_BLE_ADV_FLAG_BREDR_NOT_SPT) as u8,
        }
    }

    fn gatts_callback(&mut self, event: GattServiceEvent) -> Result<(), EspError> {
        match event {
            GattServiceEvent::Register(reg) => {
                log::info!(
                    "REGISTER_APP_EVT, status {}, app_id {}",
                    reg.status,
                    reg.app_id
                );

                let mut service_id: esp_idf_sys::esp_gatt_srvc_id_t =
                    esp_idf_sys::esp_gatt_srvc_id_t::default();
                service_id.is_primary = true;
                service_id.id.inst_id = 0x00;
                service_id.id.uuid.len = 2;
                service_id.id.uuid.uuid.uuid16 = GATTS_SERVICE_UUID_TEST_A;

                self.service_id = Some(service_id);

                esp!(unsafe { esp_idf_sys::esp_ble_gap_set_device_name(DEVICE_NAME.as_ptr(),) })?;

                //config adv data
                let adv_data = &mut BleApp::get_adv_data().clone();
                esp!(unsafe { esp_idf_sys::esp_ble_gap_config_adv_data(adv_data) })?;

                ADVERTISE_CONFIGURED.store(true, std::sync::atomic::Ordering::SeqCst);

                //config scan response data
                let scan_rsp_data = &mut BleApp::get_scan_rsp_data().clone();
                esp!(unsafe { esp_idf_sys::esp_ble_gap_config_adv_data(scan_rsp_data) })?;

                SCAN_RESPONSE_CONFIGURED.store(true, std::sync::atomic::Ordering::SeqCst);

                esp!(unsafe {
                    esp_idf_sys::esp_ble_gatts_create_service(
                        self.gatts_if,
                        self.service_id.as_mut().unwrap(),
                        GATTS_NUM_HANDLE_TEST_A,
                    )
                })?;
            }
            GattServiceEvent::Read(read) => {
                log::info!("Read {{ {:?} }}", read);
                let mut rsp: esp_idf_sys::esp_gatt_rsp_t = esp_idf_sys::esp_gatt_rsp_t::default();
                rsp.attr_value.handle = read.handle;
                rsp.attr_value.len = 4;
                unsafe {
                    rsp.attr_value.value[0] = 0xde;
                    rsp.attr_value.value[1] = 0xed;
                    rsp.attr_value.value[2] = 0xbe;
                    rsp.attr_value.value[3] = 0xef;
                }
                rsp.attr_value.auth_req = 0;
                rsp.attr_value.offset = 0;

                log::info!("Sending response");
                let res = unsafe {
                    esp_idf_sys::esp_ble_gatts_send_response(
                        self.gatts_if,
                        read.conn_id,
                        read.trans_id,
                        esp_idf_sys::esp_gatt_status_t_ESP_GATT_OK,
                        &mut rsp,
                    )
                };
                log::info!("Result: {:?}", res);
                log::info!("Response: {:?}", unsafe { rsp.attr_value });
            }
            GattServiceEvent::Write(write) => {
                log::info!("Write {{ {:?} }}", write);
                let write_content =
                    unsafe { core::slice::from_raw_parts_mut(write.value, write.len as usize) };
                log::info!("Written value({}): {:?}", write.len, write_content);
            }
            GattServiceEvent::ExecWrite(exec_write) => {
                log::info!("ExecWrite {{ {:?} }}", exec_write);
            }
            GattServiceEvent::Mtu(mtu) => {
                log::info!("ESP_GATTS_MTU_EVT, MTU {}", mtu.mtu);
            }
            GattServiceEvent::Unregister(_) => {
                log::info!("ESP_GATTS_MTU_EVT, Unregister");
            }
            GattServiceEvent::Create(create) => {
                log::info!(
                    "CREATE_SERVICE_EVT, {{ status {},  service_handle {} }}",
                    create.status,
                    create.service_handle
                );
                let mut char_uuid: esp_idf_sys::esp_bt_uuid_t =
                    esp_idf_sys::esp_bt_uuid_t::default();
                char_uuid.len = 2;
                char_uuid.uuid.uuid16 = GATTS_CHAR_UUID_TEST_A;
                self.service_handle = Some(create.service_handle);
                self.char_uuid = Some(char_uuid);

                esp!(unsafe {
                    esp_idf_sys::esp_ble_gatts_start_service(self.service_handle.unwrap())
                })?;

                let mut attr_value: [u8; 4] = [0x11, 0x22, 0x33, 0x00];
                let mut attr: esp_idf_sys::esp_attr_value_t = esp_idf_sys::esp_attr_value_t {
                    attr_max_len: 0x40,
                    attr_len: attr_value.len() as u16,
                    attr_value: attr_value.as_mut_ptr(),
                };

                self.a_property = Some(
                    (esp_idf_sys::ESP_GATT_CHAR_PROP_BIT_READ
                        | esp_idf_sys::ESP_GATT_CHAR_PROP_BIT_WRITE
                        | esp_idf_sys::ESP_GATT_CHAR_PROP_BIT_NOTIFY) as u8,
                );

                esp!(unsafe {
                    esp_idf_sys::esp_ble_gatts_add_char(
                        self.service_handle.unwrap(),
                        &mut self.char_uuid.unwrap(),
                        (esp_idf_sys::ESP_GATT_PERM_READ | esp_idf_sys::ESP_GATT_PERM_WRITE) as u16,
                        self.a_property.unwrap() as u8,
                        &mut attr,
                        std::ptr::null_mut(),
                    )
                })?;
            }
            GattServiceEvent::AddIncludedServiceComplete(add_included_service) => {
                log::info!("AddIncludedService: {{ {:?} }}", add_included_service);
            }
            GattServiceEvent::AddCharacteristicComplete(add_char) => {
                let mut length: u16 = 0;
                let prf_char: &[u8] = &[0; 40];

                log::info!(
                    "ADD_CHAR_EVT, status {},  attr_handle {}, service_handle {}",
                    add_char.status,
                    add_char.attr_handle,
                    add_char.service_handle
                );
                self.char_handle = Some(add_char.attr_handle);
                let mut descr_uuid: esp_idf_sys::esp_bt_uuid_t =
                    esp_idf_sys::esp_bt_uuid_t::default();
                descr_uuid.len = 0x2;
                descr_uuid.uuid.uuid16 = esp_idf_sys::ESP_GATT_UUID_CHAR_CLIENT_CONFIG as u16;
                self.descr_uuid = Some(descr_uuid);

                esp!(unsafe {
                    esp_idf_sys::esp_ble_gatts_get_attr_value(
                        add_char.attr_handle,
                        &mut length,
                        &mut prf_char.as_ptr(),
                    )
                })?;

                log::info!("the gatts demo char length = {}", length);
                for i in 0..length {
                    log::info!("prf_char[{}] ={}", i, prf_char[i as usize]);
                }
                esp!(unsafe {
                    esp_idf_sys::esp_ble_gatts_add_char_descr(
                        self.service_handle.unwrap(),
                        &mut self.descr_uuid.unwrap(),
                        (esp_idf_sys::ESP_GATT_PERM_READ | esp_idf_sys::ESP_GATT_PERM_WRITE) as u16,
                        std::ptr::null_mut(),
                        std::ptr::null_mut(),
                    )
                })?;
            }
            GattServiceEvent::AddDescriptorComplete(add_char_descr) => {
                self.descr_handle = Some(add_char_descr.attr_handle);
                log::info!(
                    "ADD_DESCR_EVT, status {}, attr_handle {}, service_handle {}",
                    add_char_descr.status,
                    add_char_descr.attr_handle,
                    add_char_descr.service_handle
                );
            }
            GattServiceEvent::DeleteComplete(delete) => {
                log::info!("Delete: {{ {:?} }}", delete);
            }
            GattServiceEvent::StartComplete(start) => {
                log::info!(
                    "SERVICE_START_EVT, status {}, service_handle {}",
                    start.status,
                    start.service_handle
                );
            }
            GattServiceEvent::StopComplete(stop) => {
                log::info!("Stop: {{ {:?} }}", stop);
            }
            GattServiceEvent::Connect(connect) => {
                log::info!("Connect: {{ {:?} }}", connect);
                let mut conn_params: esp_idf_sys::esp_ble_conn_update_params_t =
                    esp_idf_sys::esp_ble_conn_update_params_t::default();
                conn_params.bda = connect.remote_bda;
                /* For the IOS system, please reference the apple official documents about the ble connection parameters restrictions. */
                conn_params.latency = 0;
                conn_params.max_int = 0x20; // max_int = 0x20*1.25ms = 40ms
                conn_params.min_int = 0x10; // min_int = 0x10*1.25ms = 20ms
                conn_params.timeout = 400; // timeout = 400*10ms = 4000ms
                                           //
                log::info!("ESP_GATTS_CONNECT_EVT, conn_id {}, remote {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:",
                         connect.conn_id,
                         connect.remote_bda[0], connect.remote_bda[1], connect.remote_bda[2],
                         connect.remote_bda[3], connect.remote_bda[4], connect.remote_bda[5]);
                self.conn_id = Some(connect.conn_id);
                //start sent the update connection parameters to the peer device.
                esp!(unsafe { esp_idf_sys::esp_ble_gap_update_conn_params(&mut conn_params) })?;
            }
            GattServiceEvent::Disconnect(param) => {
                log::info!(
                    "ESP_GATTS_DISCONNECT_EVT, disconnect reason 0x{:?}",
                    param.reason
                );
                let adv_params = &mut ADV_PARAMS.clone();
                esp!(unsafe { esp_idf_sys::esp_ble_gap_start_advertising(adv_params) })?;
            }
            GattServiceEvent::Confirm(confirm) => {
                log::info!("Confirm: {{ {:?} }}", confirm);
            }
            GattServiceEvent::ResponseComplete(rsp) => {
                log::info!("Response: {{ {:?} }}", rsp);
            }
            _ => {
                log::info!("Unhandled event: {:?}", event);
            }
        }
        Ok(())
    }
}

lazy_static::lazy_static! {
    static ref BLE_SERVER: Arc<Mutex<BleServer>> = Arc::new(Mutex::new(BleServer::default()));
}

unsafe extern "C" fn gap_event_handler(
    event: esp_idf_sys::esp_gap_ble_cb_event_t,
    param: *mut esp_idf_sys::esp_ble_gap_cb_param_t,
) {
    let event: GapEvent = GapEvent::build(event, param);
    log::info!(
        "Called gap_event_handler with:\n\tevent:{:?}\n\tparam: {:?}",
        event,
        param
    );

    match event {
        GapEvent::AdvertisingDatasetComplete(_) => {
            SCAN_RESPONSE_CONFIGURED.store(false, std::sync::atomic::Ordering::SeqCst);
            if !ADVERTISE_CONFIGURED.load(std::sync::atomic::Ordering::SeqCst) {
                let adv_params = &mut ADV_PARAMS.clone();
                esp_idf_sys::esp_ble_gap_start_advertising(adv_params);
            }
        }
        GapEvent::ScanResponseDatasetComplete(_) => {
            ADVERTISE_CONFIGURED.store(false, std::sync::atomic::Ordering::SeqCst);
            if !SCAN_RESPONSE_CONFIGURED.load(std::sync::atomic::Ordering::SeqCst) {
                let adv_params = &mut ADV_PARAMS.clone();
                esp_idf_sys::esp_ble_gap_start_advertising(adv_params);
            }
        }
        GapEvent::AdvertisingStartComplete(param) => {
            //advertising start complete event to indicate advertising start successfully or failed
            if param.status != esp_idf_sys::esp_bt_status_t_ESP_BT_STATUS_SUCCESS {
                log::error!("Advertising start failed");
            }
        }
        GapEvent::AdvertisingStopComplete(param) => {
            if param.status == esp_idf_sys::esp_bt_status_t_ESP_BT_STATUS_SUCCESS {
                log::error!("Advertising stop failed");
            } else {
                log::info!("Stop adv successfully");
            }
        }
        GapEvent::UpdateConnectionParamsComplete(param) => {
            log::info!("update connection params status = {}, min_int = {}, max_int = {},conn_int = {},latency = {}, timeout = {}",
                     param.status,
                     param.min_int,
                     param.max_int,
                     param.conn_int,
                     param.latency,
                     param.timeout);
        }
        _ => {}
    }
}

unsafe extern "C" fn gatts_event_handler(
    event: esp_idf_sys::esp_gatts_cb_event_t,
    gatts_if: esp_idf_sys::esp_gatt_if_t,
    param: *mut esp_idf_sys::esp_ble_gatts_cb_param_t,
) {
    let event: GattServiceEvent = GattServiceEvent::build(event, param);
    log::info!(
        "Called gatts_event_handler with\n\tevent: {:?}\n\tgatts_if: {:?}\n\tparam: {:?}",
        event,
        gatts_if,
        param
    );

    match event {
        GattServiceEvent::Register(reg) => {
            if reg.status == esp_idf_sys::esp_gatt_status_t_ESP_GATT_OK {
                let ble_server = Arc::clone(&BLE_SERVER);
                let ble_server = ble_server.lock();
                let mut ble_server = ble_server.unwrap();

                ble_server.applications.insert(
                    gatts_if,
                    BleApp {
                        gatts_if,
                        service_id: None,
                        service_handle: None,
                        char_uuid: None,
                        char_handle: None,
                        descr_uuid: None,
                        descr_handle: None,
                        conn_id: None,
                        a_property: None,
                    },
                );
                log::info!("Registered application on gatts_if: {}", gatts_if);
            } else {
                log::error!(
                    "Register app failed: {{ status: {}, app_id: {} }}",
                    reg.status,
                    reg.app_id
                );
                return;
            }
        }
        _ => {}
    }

    let ble_server = Arc::clone(&BLE_SERVER);
    let ble_server = ble_server.lock();
    let mut ble_server = ble_server.unwrap();
    ble_server
        .applications
        .get_mut(&gatts_if)
        .and_then(|ble_app| ble_app.gatts_callback(event).ok());
}

pub fn bluetooth() -> Result<()> {
    unsafe {
        esp!(esp_idf_sys::esp_bt_controller_mem_release(
            esp_idf_sys::esp_bt_mode_t_ESP_BT_MODE_CLASSIC_BT
        ))?;

        let mut bt_cfg = bt_controller_default_config();
        esp!(esp_idf_sys::esp_bt_controller_init(&mut bt_cfg))?;

        esp!(esp_idf_sys::esp_bt_controller_enable(
            esp_idf_sys::esp_bt_mode_t_ESP_BT_MODE_BLE
        ))?;

        esp!(esp_idf_sys::esp_bluedroid_init())?;

        esp!(esp_idf_sys::esp_bluedroid_enable())?;

        esp!(esp_idf_sys::esp_ble_gatts_register_callback(Some(
            gatts_event_handler
        )))?;

        esp!(esp_idf_sys::esp_ble_gap_register_callback(Some(
            gap_event_handler
        )))?;

        esp!(esp_idf_sys::esp_ble_gatts_app_register(PROFILE_A_APP_ID))?;

        esp!(esp_idf_sys::esp_ble_gatt_set_local_mtu(500))?;
    }
    Ok(())
}
