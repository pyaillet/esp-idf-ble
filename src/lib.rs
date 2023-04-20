mod advertise;
mod gap;
mod gatt;
mod gatt_client;
mod gatt_server;
mod security;

#[macro_use]
extern crate lazy_static;

use std::ffi::c_void;
use std::{collections::HashMap, ffi::CString, sync::Arc};

use ::log::*;
use advertise::RawAdvertiseData;

use esp_idf_svc::nvs::EspDefaultNvs;

use esp_idf_sys::*;

use std::sync::Mutex;

pub use advertise::*;
pub use gap::*;
pub use gatt::*;
pub use gatt_client::*;
pub use gatt_server::*;
pub use security::*;

static DEFAULT_TAKEN: Mutex<bool> = Mutex::new(false);

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
enum GapCallbacks {
    RawAdvertisingDataset,
    RawScanResponseDataset,
    AdvertisingDataset,
    ScanResponseDataset,
    AdvertisingStart,
    UpdateConnectionParams,
    PasskeyNotify,
    KeyEvent,
    AuthComplete,
    NumericComparisonRequest,
    SecurityRequest,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
enum GattCallbacks {
    Register(u16),              // app_id
    Create(u8),                 // gatts_if
    Start(u16),                 // svc_handle
    AddCharacteristic(u16),     // svc_handle
    AddCharacteristicDesc(u16), // svc_handle
    Read(u16),                  // attr_handle
    Write(u16),                 // attr_handle
    Connect(u8),                // gatts_if
}
lazy_static! {
    static ref GAP_CALLBACKS: Mutex<HashMap<GapCallbacks, Box<dyn Fn(GapEvent) + Send>>> =
        Mutex::new(HashMap::new());
    static ref GATT_CALLBACKS_ONE_TIME: Mutex<HashMap<GattCallbacks, Box<dyn Fn(u8, GattServiceEvent) + Send>>> =
        Mutex::new(HashMap::new());
    static ref GATT_CALLBACKS_KEPT: Mutex<HashMap<GattCallbacks, Box<dyn Fn(u8, GattServiceEvent) + Send>>> =
        Mutex::new(HashMap::new());
}
fn insert_gatt_cb_kept(cb_key: GattCallbacks, cb: impl Fn(u8, GattServiceEvent) + Send + 'static) {
    GATT_CALLBACKS_KEPT
        .lock()
        .as_mut()
        .and_then(|m| Ok(m.insert(cb_key, Box::new(cb)))).unwrap();
}

fn insert_gatt_cb_onetime(
    cb_key: GattCallbacks,
    cb: impl Fn(u8, GattServiceEvent) + Send + 'static,
) {
    GATT_CALLBACKS_ONE_TIME
        .lock()
        .as_mut()
        .and_then(|m| Ok(m.insert(cb_key, Box::new(cb)))).unwrap();
}

fn insert_gap_cb(cb_key: GapCallbacks, cb: impl Fn(GapEvent) + Send + 'static) {
    GAP_CALLBACKS
        .lock()
        .as_mut()
        .and_then(|m| Ok(m.insert(cb_key, Box::new(cb)))).unwrap();
}

unsafe extern "C" fn gap_event_handler(
    event: esp_gap_ble_cb_event_t,
    param: *mut esp_ble_gap_cb_param_t,
) {
    let event = GapEvent::build(event, param);
    debug!("Called gap event handler with event {{ {:#?} }}", &event);

    if let Ok(Some(cb)) = GAP_CALLBACKS.lock().as_mut().map(|m| {
        (match &event {
            GapEvent::RawAdvertisingDatasetComplete(_) => {
                Some(&GapCallbacks::RawAdvertisingDataset)
            }
            GapEvent::RawScanResponseDatasetComplete(_) => {
                Some(&GapCallbacks::RawScanResponseDataset)
            }
            GapEvent::AdvertisingDatasetComplete(_) => Some(&GapCallbacks::AdvertisingDataset),
            GapEvent::ScanResponseDatasetComplete(_) => Some(&GapCallbacks::ScanResponseDataset),
            GapEvent::AdvertisingStartComplete(_) => Some(&GapCallbacks::AdvertisingStart),
            GapEvent::UpdateConnectionParamsComplete(_) => {
                Some(&GapCallbacks::UpdateConnectionParams)
            }
            GapEvent::PasskeyNotification(_) => Some(&GapCallbacks::PasskeyNotify),
            GapEvent::Key(_) => Some(&GapCallbacks::KeyEvent),
            GapEvent::AuthenticationComplete(_) => Some(&GapCallbacks::AuthComplete),
            GapEvent::NumericComparisonRequest(_) => Some(&GapCallbacks::NumericComparisonRequest),
            GapEvent::SecurityRequest(_) => Some(&GapCallbacks::SecurityRequest),
            _ => {
                warn!("Unimplemented {:?}", event);
                None
            }
        })
        .and_then(|cb_key| m.get(cb_key))
    }) {
        cb(event);
    } else {
        warn!("No callbak registered for event: {:?}", event);
    }
}

unsafe extern "C" fn gatts_event_handler(
    event: esp_gatts_cb_event_t,
    gatts_if: esp_gatt_if_t,
    param: *mut esp_ble_gatts_cb_param_t,
) {
    let event = GattServiceEvent::build(event, param);
    debug!(
        "Called gatt service event handler with gatts_if: {}, event {{ {:#?} }}",
        gatts_if, &event
    );

    match &event {
        GattServiceEvent::Register(reg) => {
            if let Ok(Some(cb)) = GATT_CALLBACKS_ONE_TIME
                .lock()
                .as_mut()
                .and_then(|m| Ok(m.remove(&GattCallbacks::Register(reg.app_id))))
            {
                cb(gatts_if, event);
            } else {
                warn!(
                    "No callback registered for Register with app_id: {}",
                    reg.app_id
                );
            }
        }
        GattServiceEvent::Create(_) => {
            if let Ok(Some(cb)) = GATT_CALLBACKS_ONE_TIME
                .lock()
                .as_mut()
                .and_then(|m| Ok(m.remove(&GattCallbacks::Create(gatts_if))))
            {
                cb(gatts_if, event);
            } else {
                warn!(
                    "No callback registered for Create with gatts_if: {}",
                    gatts_if
                );
            }
        }
        GattServiceEvent::StartComplete(start) => {
            if let Ok(Some(cb)) = GATT_CALLBACKS_ONE_TIME
                .lock()
                .as_mut()
                .and_then(|m| Ok(m.remove(&GattCallbacks::Start(start.service_handle))))
            {
                cb(gatts_if, event);
            } else {
                warn!(
                    "No callback registered for Start with svc_handle: {}",
                    start.service_handle
                );
            }
        }
        GattServiceEvent::AddCharacteristicComplete(add_char) => {
            if let Ok(Some(cb)) = GATT_CALLBACKS_ONE_TIME.lock().as_mut().and_then(|m| {
                Ok(m.remove(&GattCallbacks::AddCharacteristic(add_char.service_handle)))
            }) {
                cb(gatts_if, event);
            } else {
                warn!(
                    "No callback registered for AddChar with svc_handle: {}",
                    add_char.service_handle
                );
            }
        }
        GattServiceEvent::AddDescriptorComplete(add_desc) => {
            if let Ok(Some(cb)) = GATT_CALLBACKS_ONE_TIME.lock().as_mut().and_then(|m| {
                Ok(m.remove(&GattCallbacks::AddCharacteristicDesc(
                    add_desc.service_handle,
                )))
            }) {
                cb(gatts_if, event);
            } else {
                warn!(
                    "No callback registered for AddDesc with svc_handle: {}",
                    add_desc.service_handle
                );
            }
        }
        GattServiceEvent::Connect(conn) => {
            let mut conn_params: esp_ble_conn_update_params_t = esp_ble_conn_update_params_t {
                bda: conn.remote_bda,
                min_int: 0x10, // min_int = 0x10*1.25ms = 20ms
                max_int: 0x20, // max_int = 0x20*1.25ms = 40ms
                latency: 0,
                timeout: 400, // timeout = 400*10ms = 4000ms
            };
            //
            info!("Connection from: {:?}", conn);

            let _ = esp!(esp_ble_gap_update_conn_params(&mut conn_params));
            if let Ok(Some(cb)) = GATT_CALLBACKS_KEPT
                .lock()
                .as_mut()
                .and_then(|m| Ok(m.get(&GattCallbacks::Connect(gatts_if))))
            {
                cb(gatts_if, event);
            }
        }
        GattServiceEvent::Read(read) => {
            if let Ok(Some(cb)) = GATT_CALLBACKS_KEPT
                .lock()
                .as_mut()
                .and_then(|m| Ok(m.get(&GattCallbacks::Read(read.handle))))
            {
                cb(gatts_if, event);
            } else {
                warn!(
                    "No callback registered for Read with handle: {}",
                    read.handle
                );
            }
        }
        GattServiceEvent::Write(write) => {
            if let Ok(Some(cb)) = GATT_CALLBACKS_KEPT
                .lock()
                .as_mut()
                .and_then(|m| Ok(m.get(&GattCallbacks::Write(write.handle))))
            {
                cb(gatts_if, event);
            } else {
                warn!(
                    "No callback registered for Write with handle: {}",
                    write.handle
                );
            }
        }
        _ => warn!("Handler for {:?} not implemented", event),
    }
}

#[allow(dead_code)]
pub struct EspBle {
    device_name: String,
    nvs: Arc<EspDefaultNvs>,
}

impl EspBle {
    pub fn new(device_name: String, nvs: Arc<EspDefaultNvs>) -> Result<EspBle, EspError> {
        if let Ok(mut taken) = DEFAULT_TAKEN.lock() {
            if *taken {
                esp!(ESP_ERR_INVALID_STATE as i32)?;
            }
            println!("Test");
            let ble = Self::init(device_name, nvs)?;

            *taken = true;
            Ok(ble)
        } else {
            esp!(ESP_ERR_INVALID_STATE as i32)?;
            unreachable!()
        }
    }

    fn init(device_name: String, nvs: Arc<EspDefaultNvs>) -> Result<EspBle, EspError> {
        #[cfg(esp32)]
        let mut bt_cfg = esp_bt_controller_config_t {
            controller_task_stack_size: ESP_TASK_BT_CONTROLLER_STACK as _,
            controller_task_prio: ESP_TASK_BT_CONTROLLER_PRIO as _,
            hci_uart_no: BT_HCI_UART_NO_DEFAULT as _,
            hci_uart_baudrate: BT_HCI_UART_BAUDRATE_DEFAULT,
            scan_duplicate_mode: SCAN_DUPLICATE_MODE as _,
            scan_duplicate_type: SCAN_DUPLICATE_TYPE_VALUE as _,
            normal_adv_size: NORMAL_SCAN_DUPLICATE_CACHE_SIZE as _,
            mesh_adv_size: MESH_DUPLICATE_SCAN_CACHE_SIZE as _,
            send_adv_reserved_size: SCAN_SEND_ADV_RESERVED_SIZE as _,
            controller_debug_flag: CONTROLLER_ADV_LOST_DEBUG_BIT,
            mode: esp_bt_mode_t_ESP_BT_MODE_BLE as _,
            ble_max_conn: CONFIG_BTDM_CTRL_BLE_MAX_CONN_EFF as _,
            bt_max_acl_conn: CONFIG_BTDM_CTRL_BR_EDR_MAX_ACL_CONN_EFF as _,
            bt_sco_datapath: CONFIG_BTDM_CTRL_BR_EDR_SCO_DATA_PATH_EFF as _,
            auto_latency: BTDM_CTRL_AUTO_LATENCY_EFF != 0,
            bt_legacy_auth_vs_evt: BTDM_CTRL_LEGACY_AUTH_VENDOR_EVT_EFF != 0,
            bt_max_sync_conn: CONFIG_BTDM_CTRL_BR_EDR_MAX_SYNC_CONN_EFF as _,
            ble_sca: CONFIG_BTDM_BLE_SLEEP_CLOCK_ACCURACY_INDEX_EFF as _,
            pcm_role: CONFIG_BTDM_CTRL_PCM_ROLE_EFF as _,
            pcm_polar: CONFIG_BTDM_CTRL_PCM_POLAR_EFF as _,
            hli: BTDM_CTRL_HLI != 0,
            magic: ESP_BT_CONTROLLER_CONFIG_MAGIC_VAL,
            dup_list_refresh_period: SCAN_DUPL_CACHE_REFRESH_PERIOD as _
        };

        #[cfg(esp32c3)]
        let mut bt_cfg = esp_bt_controller_config_t {
            magic: esp_idf_sys::ESP_BT_CTRL_CONFIG_MAGIC_VAL,
            version: esp_idf_sys::ESP_BT_CTRL_CONFIG_VERSION,
            controller_task_stack_size: esp_idf_sys::ESP_TASK_BT_CONTROLLER_STACK as _,
            controller_task_prio: esp_idf_sys::ESP_TASK_BT_CONTROLLER_PRIO as _,
            controller_task_run_cpu: esp_idf_sys::CONFIG_BT_CTRL_PINNED_TO_CORE as _,
            bluetooth_mode: esp_idf_sys::CONFIG_BT_CTRL_MODE_EFF as _,
            ble_max_act: esp_idf_sys::CONFIG_BT_CTRL_BLE_MAX_ACT_EFF as _,
            sleep_mode: esp_idf_sys::CONFIG_BT_CTRL_SLEEP_MODE_EFF as _,
            sleep_clock: esp_idf_sys::CONFIG_BT_CTRL_SLEEP_CLOCK_EFF as _,
            ble_st_acl_tx_buf_nb: esp_idf_sys::CONFIG_BT_CTRL_BLE_STATIC_ACL_TX_BUF_NB as _,
            ble_hw_cca_check: esp_idf_sys::CONFIG_BT_CTRL_HW_CCA_EFF as _,
            ble_adv_dup_filt_max: esp_idf_sys::CONFIG_BT_CTRL_ADV_DUP_FILT_MAX as _,
            ce_len_type: esp_idf_sys::CONFIG_BT_CTRL_CE_LENGTH_TYPE_EFF as _,
            hci_tl_type: esp_idf_sys::CONFIG_BT_CTRL_HCI_TL_EFF as _,
            hci_tl_funcs: std::ptr::null_mut(),
            txant_dft: esp_idf_sys::CONFIG_BT_CTRL_TX_ANTENNA_INDEX_EFF as _,
            rxant_dft: esp_idf_sys::CONFIG_BT_CTRL_RX_ANTENNA_INDEX_EFF as _,
            txpwr_dft: esp_idf_sys::CONFIG_BT_CTRL_DFT_TX_POWER_LEVEL_EFF as _,
            cfg_mask: esp_idf_sys::CFG_NASK,
            scan_duplicate_mode: esp_idf_sys::SCAN_DUPLICATE_MODE as _,
            scan_duplicate_type: esp_idf_sys::SCAN_DUPLICATE_TYPE_VALUE as _,
            normal_adv_size: esp_idf_sys::NORMAL_SCAN_DUPLICATE_CACHE_SIZE as _,
            mesh_adv_size: esp_idf_sys::MESH_DUPLICATE_SCAN_CACHE_SIZE as _,
            coex_phy_coded_tx_rx_time_limit:
                esp_idf_sys::CONFIG_BT_CTRL_COEX_PHY_CODED_TX_RX_TLIM_EFF as _,
            hw_target_code: esp_idf_sys::BLE_HW_TARGET_CODE_ESP32C3_CHIP_ECO0 as _,
            slave_ce_len_min: esp_idf_sys::SLAVE_CE_LEN_MIN_DEFAULT as _,
            hw_recorrect_en: esp_idf_sys::AGC_RECORRECT_EN as _,
            cca_thresh: esp_idf_sys::CONFIG_BT_CTRL_HW_CCA_VAL as _,
            coex_param_en: false,
            coex_use_hooks: false,
        };

        #[cfg(esp32s3)]
        let mut bt_cfg = esp_bt_controller_config_t {
            magic: esp_idf_sys::ESP_BT_CTRL_CONFIG_MAGIC_VAL as _,
            version: esp_idf_sys::ESP_BT_CTRL_CONFIG_VERSION as _,
            controller_task_stack_size: esp_idf_sys::ESP_TASK_BT_CONTROLLER_STACK as _,
            controller_task_prio: esp_idf_sys::ESP_TASK_BT_CONTROLLER_PRIO as _,
            controller_task_run_cpu: esp_idf_sys::CONFIG_BT_CTRL_PINNED_TO_CORE as _,
            bluetooth_mode: esp_idf_sys::CONFIG_BT_CTRL_MODE_EFF as _,
            ble_max_act: esp_idf_sys::CONFIG_BT_CTRL_BLE_MAX_ACT_EFF as _,
            sleep_mode: esp_idf_sys::CONFIG_BT_CTRL_SLEEP_MODE_EFF as _,
            sleep_clock: esp_idf_sys::CONFIG_BT_CTRL_SLEEP_CLOCK_EFF as _,
            ble_st_acl_tx_buf_nb: esp_idf_sys::CONFIG_BT_CTRL_BLE_STATIC_ACL_TX_BUF_NB as _,
            ble_hw_cca_check: esp_idf_sys::CONFIG_BT_CTRL_HW_CCA_EFF as _,
            ble_adv_dup_filt_max: esp_idf_sys::CONFIG_BT_CTRL_ADV_DUP_FILT_MAX as _,
            coex_param_en: false,
            ce_len_type: esp_idf_sys::CONFIG_BT_CTRL_CE_LENGTH_TYPE_EFF as _,
            coex_use_hooks: false,
            hci_tl_type: esp_idf_sys::CONFIG_BT_CTRL_HCI_TL_EFF as _,
            hci_tl_funcs: std::ptr::null_mut(),
            txant_dft: esp_idf_sys::CONFIG_BT_CTRL_TX_ANTENNA_INDEX_EFF as _,
            rxant_dft: esp_idf_sys::CONFIG_BT_CTRL_RX_ANTENNA_INDEX_EFF as _,
            txpwr_dft: esp_idf_sys::CONFIG_BT_CTRL_DFT_TX_POWER_LEVEL_EFF as _,
            cfg_mask: esp_idf_sys::CFG_MASK as _,
            scan_duplicate_mode: esp_idf_sys::SCAN_DUPLICATE_MODE as _,
            scan_duplicate_type: esp_idf_sys::SCAN_DUPLICATE_TYPE_VALUE as _,
            normal_adv_size: esp_idf_sys::NORMAL_SCAN_DUPLICATE_CACHE_SIZE as _,
            mesh_adv_size: esp_idf_sys::MESH_DUPLICATE_SCAN_CACHE_SIZE as _,
            coex_phy_coded_tx_rx_time_limit:
                esp_idf_sys::CONFIG_BT_CTRL_COEX_PHY_CODED_TX_RX_TLIM_EFF as _,
            hw_target_code: esp_idf_sys::BLE_HW_TARGET_CODE_CHIP_ECO0 as _,
            slave_ce_len_min: esp_idf_sys::SLAVE_CE_LEN_MIN_DEFAULT as _,
            hw_recorrect_en: esp_idf_sys::AGC_RECORRECT_EN as _,
            cca_thresh: esp_idf_sys::CONFIG_BT_CTRL_HW_CCA_VAL as _,
            ble_50_feat_supp: esp_idf_sys::BT_CTRL_50_FEATURE_SUPPORT != 0,
            dup_list_refresh_period: esp_idf_sys::DUPL_SCAN_CACHE_REFRESH_PERIOD as _,
            scan_backoff_upperlimitmax: esp_idf_sys::BT_CTRL_SCAN_BACKOFF_UPPERLIMITMAX as _
        };

        info!("Init bluetooth controller.");
        esp!(unsafe { esp_bt_controller_init(&mut bt_cfg) })?;

        info!("Enable bluetooth controller.");
        esp!(unsafe { esp_bt_controller_enable(esp_bt_mode_t_ESP_BT_MODE_BLE) })?;

        info!("Init bluedroid");
        esp!(unsafe { esp_bluedroid_init() })?;

        info!("Enable bluedroid");
        esp!(unsafe { esp_bluedroid_enable() })?;

        esp!(unsafe { esp_ble_gatts_register_callback(Some(gatts_event_handler)) })?;

        esp!(unsafe { esp_ble_gap_register_callback(Some(gap_event_handler)) })?;

        esp!(unsafe { esp_ble_gatt_set_local_mtu(500) })?;

        let device_name_cstr = CString::new(device_name.clone()).unwrap();
        esp!(unsafe { esp_ble_gap_set_device_name(device_name_cstr.as_ptr() as _) })?;

        Ok(EspBle { device_name, nvs })
    }

    pub fn configure_advertising_data_raw(
        &self,
        data: RawAdvertiseData,
        cb: impl Fn(GapEvent) + 'static + Send,
    ) -> Result<(), EspError> {
        info!("configure_advertising_data_raw enter");

        let (raw_data, raw_len) = data.as_raw_data();

        insert_gap_cb(
            if data.set_scan_rsp {
                GapCallbacks::RawScanResponseDataset
            } else {
                GapCallbacks::AdvertisingDataset
            },
            cb,
        );
        if data.set_scan_rsp {
            esp!(unsafe { esp_ble_gap_config_scan_rsp_data_raw(raw_data, raw_len) })
        } else {
            esp!(unsafe { esp_ble_gap_config_adv_data_raw(raw_data, raw_len) })
        }
    }

    pub fn configure_advertising_data(
        &self,
        data: advertise::AdvertiseData,
        cb: impl Fn(GapEvent) + 'static + Send,
    ) -> Result<(), EspError> {
        info!("configure_advertising enter");

        let manufacturer_len = data.manufacturer.as_ref().map(|m| m.len()).unwrap_or(0) as u16;
        let service_data_len = data.service.as_ref().map(|s| s.len()).unwrap_or(0) as u16;
        #[repr(C, align(4))]
        struct aligned_uuid {
            uuid: [u8; 16],
        }
        let mut svc_uuid: aligned_uuid = aligned_uuid { uuid: [0; 16] };

        let svc_uuid_len = data
            .service_uuid
            .map(|bt_uuid| match bt_uuid {
                BtUuid::Uuid16(uuid) => {
                    svc_uuid.uuid[0..2].copy_from_slice(&uuid.to_le_bytes());
                    2
                }
                BtUuid::Uuid32(uuid) => {
                    svc_uuid.uuid[0..4].copy_from_slice(&uuid.to_le_bytes());
                    4
                }
                BtUuid::Uuid128(uuid) => {
                    svc_uuid.uuid.copy_from_slice(&uuid);
                    16
                }
            })
            .unwrap_or(0);

        let is_scan_rsp = data.set_scan_rsp;

        info!("svc_uuid: {{ {:?} }}", &svc_uuid.uuid);
        let mut adv_data = esp_ble_adv_data_t {
            set_scan_rsp: data.set_scan_rsp,
            include_name: data.include_name,
            include_txpower: data.include_txpower,
            min_interval: data.min_interval,
            max_interval: data.max_interval,
            manufacturer_len,
            p_manufacturer_data: data
                .manufacturer
                .map_or(std::ptr::null_mut(), |mut m| m.as_mut_ptr()),
            service_data_len,
            p_service_data: data
                .service
                .map_or(std::ptr::null_mut(), |mut s| s.as_mut_ptr()),
            service_uuid_len: svc_uuid_len,
            p_service_uuid: if svc_uuid_len == 0 {
                std::ptr::null_mut()
            } else {
                let ptr = svc_uuid.uuid.as_mut_ptr();
                unsafe {
                    info!("0:{:0x}", *ptr as u8);
                    info!("1:{:0x}", *ptr.add(1) as u8);
                    info!("2:{:0x}", *ptr.add(2) as u8);
                    info!("3:{:0x}", *ptr.add(3) as u8);
                }
                ptr
            },
            appearance: data.appearance.into(),
            flag: data.flag,
        };

        if is_scan_rsp {
            insert_gap_cb(GapCallbacks::ScanResponseDataset, cb);
        } else {
            insert_gap_cb(GapCallbacks::AdvertisingDataset, cb);
        };

        info!("Configuring advertising with {{ {:?} }}", &adv_data);

        esp!(unsafe { esp_ble_gap_config_adv_data(&mut adv_data) })
    }

    pub fn start_advertise(&self, cb: impl Fn(GapEvent) + 'static + Send) -> Result<(), EspError> {
        info!("start_advertise enter");

        let mut adv_param: esp_ble_adv_params_t = esp_ble_adv_params_t {
            adv_int_min: 0x20,
            adv_int_max: 0x40,
            adv_type: 0x00,      // ADV_TYPE_IND,
            own_addr_type: 0x00, // BLE_ADDR_TYPE_PUBLIC,
            peer_addr: [0; 6],
            peer_addr_type: 0x00,    // BLE_ADDR_TYPE_PUBLIC,
            channel_map: 0x07,       // ADV_CHNL_ALL,
            adv_filter_policy: 0x00, // ADV_FILTER_ALLOW_SCAN_ANY_CON_ANY,
        };

        insert_gap_cb(GapCallbacks::AdvertisingStart, cb);
        esp!(unsafe { esp_ble_gap_start_advertising(&mut adv_param) })
    }

    pub fn register_gatt_service_application(
        &mut self,
        app_id: u16,
        cb: impl Fn(u8, GattServiceEvent) + 'static + Send,
    ) -> Result<(), EspError> {
        info!(
            "register_gatt_service_application enter for app_id: {}",
            app_id
        );
        insert_gatt_cb_onetime(GattCallbacks::Register(app_id), cb);
        esp!(unsafe { esp_ble_gatts_app_register(app_id) })
    }

    pub fn create_service(
        &self,
        gatt_if: u8,
        svc: GattService,
        cb: impl Fn(u8, GattServiceEvent) + 'static + Send,
    ) -> Result<(), EspError> {
        let svc_uuid: esp_bt_uuid_t = svc.id.into();

        let mut svc_id: esp_gatt_srvc_id_t = esp_gatt_srvc_id_t {
            is_primary: svc.is_primary,
            id: esp_gatt_id_t {
                uuid: svc_uuid,
                inst_id: svc.instance_id,
            },
        };
        insert_gatt_cb_onetime(GattCallbacks::Create(gatt_if), cb);

        esp!(unsafe { esp_ble_gatts_create_service(gatt_if, &mut svc_id, svc.handle) })
    }

    pub fn start_service(
        &self,
        svc_handle: u16,
        cb: impl Fn(u8, GattServiceEvent) + 'static + Send,
    ) -> Result<(), EspError> {
        insert_gatt_cb_onetime(GattCallbacks::Start(svc_handle), cb);

        esp!(unsafe { esp_ble_gatts_start_service(svc_handle) })
    }

    pub fn read_attribute_value(&self, attr_handle: u16) -> Result<Vec<u8>, EspError> {
        let mut len: u16 = 0;
        let mut data: *const u8 = std::ptr::null_mut();

        unsafe {
            esp!(esp_ble_gatts_get_attr_value(
                attr_handle,
                &mut len,
                &mut data
            ))?;

            let data = std::slice::from_raw_parts(data, len as usize);
            info!("len: {:?}, data: {:p}", len, data);
            Ok(data.to_vec())
        }
    }

    pub fn add_characteristic<const S: usize>(
        &self,
        svc_handle: u16,
        charac: GattCharacteristic<S>,
        cb: impl Fn(u8, GattServiceEvent) + 'static + Send,
    ) -> Result<(), EspError> {
        insert_gatt_cb_onetime(GattCallbacks::AddCharacteristic(svc_handle), cb);

        let mut uuid = charac.uuid.into();

        let mut value = charac.value.into();
        let mut auto_rsp = charac.auto_rsp.into();

        esp!(unsafe {
            esp_ble_gatts_add_char(
                svc_handle,
                &mut uuid,
                charac.permissions,
                charac.property,
                &mut value,
                &mut auto_rsp,
            )
        })
    }

    pub fn add_descriptor(
        &self,
        svc_handle: u16,
        char_desc: GattDescriptor,
        cb: impl Fn(u8, GattServiceEvent) + 'static + Send,
    ) -> Result<(), EspError> {
        insert_gatt_cb_onetime(GattCallbacks::AddCharacteristicDesc(svc_handle), cb);

        let mut uuid = char_desc.uuid.into();

        esp!(unsafe {
            esp_ble_gatts_add_char_descr(
                svc_handle,
                &mut uuid,
                char_desc.permissions,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        })
    }

    pub fn register_connect_handler(
        &self,
        gatts_if: u8,
        cb: impl Fn(u8, GattServiceEvent) + 'static + Send,
    ) {
        insert_gatt_cb_kept(GattCallbacks::Connect(gatts_if), cb);
    }

    pub fn register_read_handler(
        &self,
        attr_handle: u16,
        cb: impl Fn(u8, GattServiceEvent) + 'static + Send,
    ) {
        insert_gatt_cb_kept(GattCallbacks::Read(attr_handle), cb);
    }

    pub fn register_write_handler(
        &self,
        attr_handle: u16,
        cb: impl Fn(u8, GattServiceEvent) + 'static + Send,
    ) {
        insert_gatt_cb_kept(GattCallbacks::Write(attr_handle), cb);
    }

    pub fn configure_security(&self, mut config: SecurityConfig) -> Result<(), EspError> {
        esp!(unsafe {
            esp_ble_gap_set_security_param(
                esp_ble_sm_param_t_ESP_BLE_SM_AUTHEN_REQ_MODE,
                &mut config.auth_req_mode as *mut _ as *mut c_void,
                std::mem::size_of::<u8>() as _,
            )
        })
        .expect("auth req mode");

        esp!(unsafe {
            esp_ble_gap_set_security_param(
                esp_ble_sm_param_t_ESP_BLE_SM_IOCAP_MODE,
                &mut config.io_capabilities as *mut _ as *mut c_void,
                std::mem::size_of::<u8>() as _,
            )
        })
        .expect("sm iocap mode");

        if let Some(mut initiator_key) = config.initiator_key {
            esp!(unsafe {
                esp_ble_gap_set_security_param(
                    esp_ble_sm_param_t_ESP_BLE_SM_SET_INIT_KEY,
                    &mut initiator_key as *mut _ as *mut c_void,
                    std::mem::size_of::<u8>() as _,
                )
            })
            .expect("initiator_key");
        }

        if let Some(mut responder_key) = config.responder_key {
            esp!(unsafe {
                esp_ble_gap_set_security_param(
                    esp_ble_sm_param_t_ESP_BLE_SM_SET_RSP_KEY,
                    &mut responder_key as *mut _ as *mut c_void,
                    std::mem::size_of::<u8>() as _,
                )
            })
            .expect("responder_key");
        }
        if let Some(mut max_key_size) = config.max_key_size {
            esp!(unsafe {
                esp_ble_gap_set_security_param(
                    esp_ble_sm_param_t_ESP_BLE_SM_MAX_KEY_SIZE,
                    &mut max_key_size as *mut _ as *mut c_void,
                    std::mem::size_of::<u8>() as _,
                )
            })
            .expect("max key size");
        }
        if let Some(mut min_key_size) = config.min_key_size {
            esp!(unsafe {
                esp_ble_gap_set_security_param(
                    esp_ble_sm_param_t_ESP_BLE_SM_MIN_KEY_SIZE,
                    &mut min_key_size as *mut _ as *mut c_void,
                    std::mem::size_of::<u8>() as _,
                )
            })
            .expect("min key size");
        }
        if let Some(passkey) = config.static_passkey {
            let mut passkey = passkey.to_ne_bytes();
            esp!(unsafe {
                esp_ble_gap_set_security_param(
                    esp_ble_sm_param_t_ESP_BLE_SM_SET_STATIC_PASSKEY,
                    &mut passkey as *mut _ as *mut c_void,
                    std::mem::size_of::<u32>() as _,
                )
            })
            .expect("set static passkey");
        }
        esp!(unsafe {
            let mut only_accept_specified_auth = u8::from(config.only_accept_specified_auth);
            esp_ble_gap_set_security_param(
                esp_ble_sm_param_t_ESP_BLE_SM_ONLY_ACCEPT_SPECIFIED_SEC_AUTH,
                &mut only_accept_specified_auth as *mut _ as *mut c_void,
                std::mem::size_of::<u8>() as _,
            )
        })
        .expect("only accept spec auth");
        esp!(unsafe {
            let mut enable_oob = u8::from(config.enable_oob);
            esp_ble_gap_set_security_param(
                esp_ble_sm_param_t_ESP_BLE_SM_OOB_SUPPORT,
                &mut enable_oob as *mut _ as *mut c_void,
                std::mem::size_of::<u8>() as _,
            )
        })
        .expect("oob support");

        insert_gap_cb(GapCallbacks::SecurityRequest, |sec_req| {
            if let GapEvent::SecurityRequest(mut sec_req) = sec_req {
                info!("SecurityRequest");
                match esp!(unsafe {
                    esp_ble_gap_security_rsp(sec_req.ble_req.bd_addr.as_mut_ptr(), true)
                }) {
                    Ok(()) => info!("Security set"),
                    Err(err) => warn!("Error setting security: {}", err),
                }
            }
        });
        insert_gap_cb(GapCallbacks::PasskeyNotify, |notify| {
            if let GapEvent::PasskeyNotification(notify) = notify {
                info!("Passkey: {:?}", unsafe { notify.ble_key.key_type })
            }
        });
        insert_gap_cb(GapCallbacks::KeyEvent, |key| {
            if let GapEvent::Key(key) = key {
                info!("Key: {:?}", unsafe { key.ble_key.key_type });
            }
        });
        insert_gap_cb(GapCallbacks::AuthComplete, |auth| {
            if let GapEvent::AuthenticationComplete(auth) = auth {
                info!("Auth: {:?}", unsafe { auth.auth_cmpl.success });
            }
        });

        insert_gap_cb(GapCallbacks::SecurityRequest, |sec_req| {
            if let GapEvent::SecurityRequest(sec_req) = sec_req {
                let mut ble_sec_req: esp_ble_sec_req_t = unsafe { sec_req.ble_req };
                info!("SecurityRequest: {:?}", ble_sec_req);
                unsafe { esp_ble_gap_security_rsp(ble_sec_req.bd_addr.as_mut_ptr(), true) };
            }
        });

        insert_gap_cb(GapCallbacks::NumericComparisonRequest, |ble_sec| {
            info!("Numeric comparison request");
            if let GapEvent::NumericComparisonRequest(mut ble_sec) = ble_sec {
                esp!(unsafe { esp_ble_confirm_reply(ble_sec.ble_req.bd_addr.as_mut_ptr(), true) })
                    .expect("Unable to complete numeric comparison request");
            }
        });

        Ok(())
    }

    pub fn configure_gatt_encryption(
        mut remote_bda: [u8; ESP_BD_ADDR_LEN as _],
        encryption_config: BleEncryption,
    ) {
        esp!(unsafe { esp_ble_set_encryption(remote_bda.as_mut_ptr(), encryption_config as u32) })
            .expect("Unable to set security level");
    }
}

pub fn send(
    gatts_if: u8,
    handle: u16,
    conn_id: u16,
    trans_id: u32,
    status: u32,
    data: &[u8],
) -> Result<(), EspError> {
    let mut rsp: esp_gatt_rsp_t = esp_gatt_rsp_t::default();

    esp!(unsafe {
        rsp.handle = handle;
        rsp.attr_value.len = data.len() as u16;
        if !data.is_empty() {
            rsp.attr_value.value[..data.len()].copy_from_slice(data);
        }

        esp_ble_gatts_send_response(gatts_if, conn_id, trans_id, status, &mut rsp)
    })
}
