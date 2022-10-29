use std::{ffi::CString, sync::{Arc, Mutex}};

use esp_idf_svc::nvs::EspDefaultNvs;
use esp_idf_sys::*;

use log::*;

static DEFAULT_TAKEN: Mutex<bool> = Mutex::new(false);

struct BLE {
    device_name: String,
    nvs: Arc<EspDefaultNvs>
}

impl BLE {
    pub fn new(device_name: String, nvs: Arc<EspDefaultNvs>) -> Result<BLE, EspError> {
        let mut taken = DEFAULT_TAKEN.lock().unwrap();

        if *taken {
            esp!(ESP_ERR_INVALID_STATE as i32)?;
        }

        let ble = Self::init(device_name, nvs)?;

        *taken = true;
        Ok(ble)
    }

    fn init(device_name: String, nvs: Arc<EspDefaultNvs>) -> Result<BLE, EspError> {
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
            hw_target_code: esp_idf_sys::BLE_HW_TARGET_CODE_ESP32S3_CHIP_ECO0 as _,
            slave_ce_len_min: esp_idf_sys::SLAVE_CE_LEN_MIN_DEFAULT as _,
            hw_recorrect_en: esp_idf_sys::AGC_RECORRECT_EN as _,
            cca_thresh: esp_idf_sys::CONFIG_BT_CTRL_HW_CCA_VAL as _,
        };

        esp!(unsafe { esp_bt_controller_init(&mut bt_cfg) })?;

        esp!(unsafe { esp_bt_controller_enable(esp_bt_mode_t_ESP_BT_MODE_BLE) })?;

        info!("init bluetooth");
        esp!(unsafe { esp_bluedroid_init() })?;

        esp!(unsafe { esp_bluedroid_enable() })?;

        // esp!(unsafe { esp_ble_gatts_register_callback(Some(gatts_event_handler)) })?;

        // esp!(unsafe { esp_ble_gap_register_callback(Some(gap_event_handler)) })?;

        esp!(unsafe { esp_ble_gatt_set_local_mtu(500) })?;

        let device_name_cstr = CString::new(device_name.clone()).unwrap();
        esp!(unsafe { esp_ble_gap_set_device_name(device_name_cstr.as_ptr() as _) })?;

        Ok(BLE { device_name, nvs })
    }
}
