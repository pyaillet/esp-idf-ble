pub mod advertise;
pub mod gap;
pub mod gatt;
pub mod gatt_client;
pub mod gatt_server;

use std::{collections::HashMap, ffi::CString, sync::Arc};

use ::log::*;

use esp_idf_svc::nvs::EspDefaultNvs;

use esp_idf_sys::*;

use esp_idf_hal::mutex::Mutex;

pub use gap::*;
pub use gatt::*;
pub use gatt_client::*;
pub use gatt_server::*;

use smol::{channel::Sender, future::block_on};

#[allow(unused)]
#[derive(PartialEq, Eq, Hash)]
enum GapCallbackType {
    AdvertisingDatasetComplete,
    ScanResponseDatasetComplete,
    AdvertisingStartComplete,
}

#[allow(unused)]
#[derive(PartialEq, Eq, Hash)]
enum GattCallbackType {
    Register(u16),
}

static DEFAULT_TAKEN: Mutex<bool> = Mutex::new(false);

type Singleton<T> = Mutex<Option<T>>;

static GAP_ADV_CONF_DATA: Singleton<Sender<Result<(), EspError>>> = Mutex::new(Option::None);
static GAP_ADV_SCAN_RSP_DATA: Singleton<Sender<Result<(), EspError>>> = Mutex::new(Option::None);
static GAP_ADV_START: Singleton<Sender<Result<(), EspError>>> = Mutex::new(Option::None);
static GATTS_REG_APP: Singleton<HashMap<u16, Sender<Result<esp_gatt_if_t, EspError>>>> =
    Mutex::new(Option::None);
static GATTS_CREATE_SVC: Singleton<HashMap<esp_gatt_if_t, Sender<Result<u16, EspError>>>> =
    Mutex::new(Option::None);
static GATTS_START_SVC: Singleton<HashMap<u16, Sender<Result<(), EspError>>>> =
    Mutex::new(Option::None);
static GATTS_ADD_CHAR: Singleton<HashMap<u16, Sender<Result<(), EspError>>>> =
    Mutex::new(Option::None);
static GATTS_ADD_CDESC: Singleton<HashMap<u16, Sender<Result<(), EspError>>>> =
    Mutex::new(Option::None);

macro_rules! event_send {
    ($event:expr, $send:expr, $param:expr) => {
        if let Some(sender) = $send.lock().as_mut().take() {
            if sender.send($param).await.is_err() {
                error!("Error sending event: {:?}", $event);
            }
        } else {
            warn!("No sender registered for: {:?}", $event);
        }
    };
}

unsafe extern "C" fn gap_event_handler(
    event: esp_gap_ble_cb_event_t,
    param: *mut esp_ble_gap_cb_param_t,
) {
    let event = GapEvent::build(event, param);
    info!("Called gap event handler with event {{ {:#?} }}", &event);

    block_on(async {
        match event {
            GapEvent::AdvertisingDatasetComplete(adv) => {
                event_send!(event, GAP_ADV_CONF_DATA, esp!(adv.status));
            }
            GapEvent::ScanResponseDatasetComplete(rsp) => {
                event_send!(event, GAP_ADV_SCAN_RSP_DATA, esp!(rsp.status));
            }
            GapEvent::AdvertisingStartComplete(start) => {
                event_send!(event, GAP_ADV_START, esp!(start.status));
            }
            _ => warn!("Unhandled event"),
        }
    });
}

unsafe extern "C" fn gatts_event_handler(
    event: esp_gatts_cb_event_t,
    gatts_if: esp_gatt_if_t,
    param: *mut esp_ble_gatts_cb_param_t,
) {
    let event = GattServiceEvent::build(event, param);
    info!(
        "Called gatt service event handler with gatts_if: {}, event {{ {:#?} }}",
        gatts_if, &event
    );

    block_on(async {
        match event {
            GattServiceEvent::Register(register) => {
                let param = esp!(register.status).and(Ok(gatts_if));
                if let Some(s) = GATTS_REG_APP
                    .lock()
                    .as_mut()
                    .map(|m| m.remove(&register.app_id))
                    .flatten()
                {
                    if s.send(param).await.is_err() {
                        error!("Error sending event: {:?}", event);
                    };
                } else {
                    warn!("No sender registered for: {:?}", event);
                }
            }
            GattServiceEvent::Create(create) => {
                if let Some(s) = GATTS_CREATE_SVC
                    .lock()
                    .as_mut()
                    .map(|m| m.remove(&gatts_if))
                    .flatten()
                {
                    if s.send(esp!(create.status).map(|_| create.service_handle))
                        .await
                        .is_err()
                    {
                        error!("Error sending event: {:?}", event);
                    };
                } else {
                    warn!(
                        "No sender registered for: {:?} {{ gatts_if: {}, handle: {} }}",
                        event, gatts_if, create.service_handle
                    );
                }
            }
            GattServiceEvent::StartComplete(start) => {
                if let Some(s) = GATTS_START_SVC
                    .lock()
                    .as_mut()
                    .map(|m| m.remove(&start.service_handle))
                    .flatten()
                {
                    if s.send(esp!(start.status)).await.is_err() {
                        error!("Error sending event: {:?}", event);
                    };
                } else {
                    warn!(
                        "No sender registered for: {:?} {{ gatts_if: {}, handle: {} }}",
                        event, gatts_if, start.service_handle
                    );
                }
            }
            GattServiceEvent::AddCharacteristicComplete(add) => {
                if let Some(s) = GATTS_ADD_CHAR
                    .lock()
                    .as_mut()
                    .map(|m| m.remove(&add.service_handle))
                    .flatten()
                {
                    if s.send(esp!(add.status)).await.is_err() {
                        error!("Error sending event: {:?}", event);
                    };
                } else {
                    warn!(
                        "No sender registered for: {:?} {{ gatts_if: {}, handle: {} }}",
                        event, gatts_if, add.service_handle
                    );
                }
            }
            GattServiceEvent::AddDescriptorComplete(add) => {
                if let Some(s) = GATTS_ADD_CDESC
                    .lock()
                    .as_mut()
                    .map(|m| m.remove(&add.service_handle))
                    .flatten()
                {
                    if s.send(esp!(add.status)).await.is_err() {
                        error!("Error sending event: {:?}", event);
                    };
                } else {
                    warn!(
                        "No sender registered for: {:?} {{ gatts_if: {}, handle: {} }}",
                        event, gatts_if, add.service_handle
                    );
                }
            }
            _ => warn!("Unhandled event"),
        }
    })
}

trait BleChar {}

#[allow(dead_code)]
pub struct EspBle {
    device_name: String,
    nvs: Arc<EspDefaultNvs>,
    applications: HashMap<esp_gatt_if_t, Arc<Mutex<GattApplication>>>,
}

impl EspBle {
    pub fn new(device_name: String, nvs: Arc<EspDefaultNvs>) -> Result<EspBle, EspError> {
        let mut taken = DEFAULT_TAKEN.lock();

        if *taken {
            esp!(ESP_ERR_INVALID_STATE as i32)?;
        }

        let ble = Self::init(device_name, nvs)?;

        *GATTS_REG_APP.lock() = Some(HashMap::new());
        *GATTS_CREATE_SVC.lock() = Some(HashMap::new());
        *GATTS_START_SVC.lock() = Some(HashMap::new());
        *GATTS_ADD_CHAR.lock() = Some(HashMap::new());

        *taken = true;
        Ok(ble)
    }

    fn init(device_name: String, nvs: Arc<EspDefaultNvs>) -> Result<EspBle, EspError> {
        #[cfg(not(esp32c3))]
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
        };

        #[cfg(esp32c3)]
        let mut bt_cfg = esp_bt_controller_config_t {
            magic: esp_idf_sys::ESP_BT_CTRL_CONFIG_MAGIC_VAL,
            version: esp_idf_sys::ESP_BT_CTRL_CONFIG_VERSION,
            controller_task_stack_size: esp_idf_sys::ESP_TASK_BT_CONTROLLER_STACK as u16,
            controller_task_prio: esp_idf_sys::ESP_TASK_BT_CONTROLLER_PRIO as u8,
            controller_task_run_cpu: esp_idf_sys::CONFIG_BT_CTRL_PINNED_TO_CORE as u8,
            bluetooth_mode: esp_idf_sys::CONFIG_BT_CTRL_MODE_EFF as u8,
            ble_max_act: esp_idf_sys::CONFIG_BT_CTRL_BLE_MAX_ACT_EFF as u8,
            sleep_mode: esp_idf_sys::CONFIG_BT_CTRL_SLEEP_MODE_EFF as u8,
            sleep_clock: esp_idf_sys::CONFIG_BT_CTRL_SLEEP_CLOCK_EFF as u8,
            ble_st_acl_tx_buf_nb: esp_idf_sys::CONFIG_BT_CTRL_BLE_STATIC_ACL_TX_BUF_NB as u8,
            ble_hw_cca_check: esp_idf_sys::CONFIG_BT_CTRL_HW_CCA_EFF as u8,
            ble_adv_dup_filt_max: esp_idf_sys::CONFIG_BT_CTRL_ADV_DUP_FILT_MAX as u16,
            ce_len_type: esp_idf_sys::CONFIG_BT_CTRL_CE_LENGTH_TYPE_EFF as u8,
            hci_tl_type: esp_idf_sys::CONFIG_BT_CTRL_HCI_TL_EFF as u8,
            hci_tl_funcs: std::ptr::null_mut(),
            txant_dft: esp_idf_sys::CONFIG_BT_CTRL_TX_ANTENNA_INDEX_EFF as u8,
            rxant_dft: esp_idf_sys::CONFIG_BT_CTRL_RX_ANTENNA_INDEX_EFF as u8,
            txpwr_dft: esp_idf_sys::CONFIG_BT_CTRL_DFT_TX_POWER_LEVEL_EFF as u8,
            cfg_mask: esp_idf_sys::CFG_NASK,
            scan_duplicate_mode: esp_idf_sys::SCAN_DUPLICATE_MODE as u8,
            scan_duplicate_type: esp_idf_sys::SCAN_DUPLICATE_TYPE_VALUE as u8,
            normal_adv_size: esp_idf_sys::NORMAL_SCAN_DUPLICATE_CACHE_SIZE as u16,
            mesh_adv_size: esp_idf_sys::MESH_DUPLICATE_SCAN_CACHE_SIZE as u16,
            coex_phy_coded_tx_rx_time_limit:
                esp_idf_sys::CONFIG_BT_CTRL_COEX_PHY_CODED_TX_RX_TLIM_EFF as u8,
            hw_target_code: esp_idf_sys::BLE_HW_TARGET_CODE_ESP32C3_CHIP_ECO0 as u32,
            slave_ce_len_min: esp_idf_sys::SLAVE_CE_LEN_MIN_DEFAULT as u8,
            hw_recorrect_en: esp_idf_sys::AGC_RECORRECT_EN as u8,
            cca_thresh: esp_idf_sys::CONFIG_BT_CTRL_HW_CCA_VAL as u8,
            coex_param_en: false,
            coex_use_hooks: false,
        };

        esp!(unsafe { esp_bt_controller_init(&mut bt_cfg) })?;

        esp!(unsafe { esp_bt_controller_enable(esp_bt_mode_t_ESP_BT_MODE_BLE) })?;

        info!("init bluetooth");
        esp!(unsafe { esp_bluedroid_init() })?;

        esp!(unsafe { esp_bluedroid_enable() })?;

        esp!(unsafe { esp_ble_gatts_register_callback(Some(gatts_event_handler)) })?;

        esp!(unsafe { esp_ble_gap_register_callback(Some(gap_event_handler)) })?;

        esp!(unsafe { esp_ble_gatt_set_local_mtu(500) })?;

        let device_name_cstr = CString::new(device_name.clone()).unwrap();
        esp!(unsafe { esp_ble_gap_set_device_name(device_name_cstr.as_ptr() as _) })?;

        Ok(EspBle {
            device_name,
            nvs,
            applications: HashMap::new(),
        })
    }

    pub async fn configure_advertising_data(
        &self,
        data: advertise::AdvertiseData,
    ) -> Result<(), EspError> {
        info!("configure_advertising enter");

        let (s, r) = smol::channel::bounded(1);

        if data.set_scan_rsp {
            *GAP_ADV_SCAN_RSP_DATA.lock() = Some(s);
        } else {
            *GAP_ADV_CONF_DATA.lock() = Some(s);
        }

        let mut adv_data: esp_ble_adv_data_t = data.try_into()?;

        esp!(unsafe { esp_ble_gap_config_adv_data(&mut adv_data) })?;

        info!("configure_advertising exit");

        r.recv().await.unwrap_or(esp!(ESP_ERR_INVALID_STATE))
    }

    pub async fn start_advertise(&self) -> Result<(), EspError> {
        info!("start_advertise enter");

        let mut adv_param: esp_ble_adv_params_t = esp_ble_adv_params_t {
            adv_int_min: 0x20,
            adv_int_max: 0x40,
            adv_type: 0x00,
            own_addr_type: 0x00,
            peer_addr: [0; 6],
            peer_addr_type: 0,
            channel_map: 0x07,
            adv_filter_policy: 0x00,
        };
        let (s, r) = smol::channel::bounded(1);

        *GAP_ADV_START.lock() = Some(s);

        esp!(unsafe { esp_ble_gap_start_advertising(&mut adv_param) })?;

        info!("start_advertise exit");
        r.recv().await.unwrap_or(esp!(ESP_ERR_INVALID_STATE))
    }

    pub async fn register_gatt_service_application(
        &mut self,
        application: Arc<Mutex<gatt_server::GattApplication>>,
    ) -> Result<(), EspError> {
        info!("register_gatt_service_application enter");
        let application_id: u16 = application.lock().get_id();

        let (s, r) = smol::channel::bounded(1);

        GATTS_REG_APP
            .lock()
            .as_mut()
            .and_then(|m| m.insert(application_id, s));

        esp!(unsafe { esp_ble_gatts_app_register(application_id) })?;

        let gatt_if = match r.recv().await {
            Ok(r) => r,
            Err(_) => Err(EspError::from(ESP_ERR_INVALID_STATE).unwrap()),
        }?;

        (*application.lock()).register(gatt_if);
        self.applications.insert(gatt_if, application);

        info!("register_gatt_service_application exit");

        Ok(())
    }

    pub async fn create_service(
        &self,
        application: Arc<Mutex<gatt_server::GattApplication>>,
        svc: GattService,
    ) -> Result<u16, EspError> {
        let gatt_if = application.lock().get_gatt_if()?;
        let mut svc_id: esp_gatt_srvc_id_t = esp_gatt_srvc_id_t {
            is_primary: svc.is_primary,
            id: esp_gatt_id_t {
                uuid: svc.id.into(),
                inst_id: svc.instance_id,
            },
        };

        let (s, r) = smol::channel::bounded(1);

        GATTS_CREATE_SVC
            .lock()
            .as_mut()
            .and_then(|m| m.insert(gatt_if, s));

        esp!(unsafe { esp_ble_gatts_create_service(gatt_if, &mut svc_id, svc.handle) })?;

        match r.recv().await {
            Ok(r) => r,
            Err(_) => Err(EspError::from(ESP_ERR_INVALID_STATE).unwrap()),
        }
    }

    pub async fn start_service(&self, svc_handle: u16) -> Result<(), EspError> {
        let (s, r) = smol::channel::bounded(1);

        GATTS_START_SVC
            .lock()
            .as_mut()
            .and_then(|m| m.insert(svc_handle, s));

        esp!(unsafe { esp_ble_gatts_start_service(svc_handle) })?;

        r.recv().await.unwrap_or(esp!(ESP_ERR_INVALID_STATE))
    }

    pub async fn add_characteristic<const S: usize>(
        &self,
        svc_handle: u16,
        charac: GattCharacteristic<S>,
    ) -> Result<(), EspError> {
        let (s, r) = smol::channel::bounded(1);

        GATTS_ADD_CHAR
            .lock()
            .as_mut()
            .and_then(|m| m.insert(svc_handle, s));

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
        })?;

        r.recv().await.unwrap_or(esp!(ESP_ERR_INVALID_STATE))
    }

    pub async fn add_characteristic_desc(
        &self,
        svc_handle: u16,
        char_desc: GattCharacteristicDesc,
    ) -> Result<(), EspError> {
        let (s, r) = smol::channel::bounded(1);

        GATTS_ADD_CDESC
            .lock()
            .as_mut()
            .and_then(|m| m.insert(svc_handle, s));

        let mut uuid = char_desc.uuid.into();

        esp!(unsafe {
            esp_ble_gatts_add_char_descr(
                svc_handle,
                &mut uuid,
                char_desc.permissions,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        })?;

        r.recv().await.unwrap_or(esp!(ESP_ERR_INVALID_STATE))
    }
}
