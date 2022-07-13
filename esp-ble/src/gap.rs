use esp_idf_sys::*;

#[derive(Clone, Copy)]
pub enum GapEvent {
    AdvertisingDatasetComplete(esp_ble_gap_cb_param_t_ble_adv_data_cmpl_evt_param),
    ScanResponseDatasetComplete(esp_ble_gap_cb_param_t_ble_scan_rsp_data_cmpl_evt_param),
    ScanParameterDatasetComplete(esp_ble_gap_cb_param_t_ble_scan_param_cmpl_evt_param),
    ScanResult(esp_ble_gap_cb_param_t_ble_scan_result_evt_param),
    RawAdvertisingDatasetComplete(esp_ble_gap_cb_param_t_ble_adv_data_raw_cmpl_evt_param),
    RawScanResponseDatasetComplete(esp_ble_gap_cb_param_t_ble_scan_rsp_data_raw_cmpl_evt_param),
    AdvertisingStartComplete(esp_ble_gap_cb_param_t_ble_adv_start_cmpl_evt_param),
    ScanStartComplete(esp_ble_gap_cb_param_t_ble_scan_start_cmpl_evt_param),
    AuthenticationComplete(esp_ble_sec_t),
    Key(esp_ble_sec_t),
    SecurityRequest(esp_ble_sec_t),
    PasskeyNotification(esp_ble_sec_t),
    PasskeyRequest(esp_ble_sec_t),
    OOBRequest,
    LocalIR,
    LocalER,
    NumericComparisonRequest,
    AdvertisingStopComplete(esp_ble_gap_cb_param_t_ble_adv_stop_cmpl_evt_param),
    ScanStopComplete(esp_ble_gap_cb_param_t_ble_scan_stop_cmpl_evt_param),
    SetStaticRandomAddressComplete(esp_ble_gap_cb_param_t_ble_set_rand_cmpl_evt_param),
    UpdateConnectionParamsComplete(esp_ble_gap_cb_param_t_ble_update_conn_params_evt_param),
    SetPacketLengthComplete(esp_ble_gap_cb_param_t_ble_pkt_data_length_cmpl_evt_param),
    SetLocalPrivacy(esp_ble_gap_cb_param_t_ble_local_privacy_cmpl_evt_param),
    RemoveDeviceBondComplete(esp_ble_gap_cb_param_t_ble_remove_bond_dev_cmpl_evt_param),
    ClearDeviceBondComplete(esp_ble_gap_cb_param_t_ble_clear_bond_dev_cmpl_evt_param),
    GetDeviceBondComplete(esp_ble_gap_cb_param_t_ble_get_bond_dev_cmpl_evt_param),
    ReadRssiComplete(esp_ble_gap_cb_param_t_ble_read_rssi_cmpl_evt_param),
    UpdateWhitelistComplete(esp_ble_gap_cb_param_t_ble_update_whitelist_cmpl_evt_param),
    UpdateDuplicateListComplete(
        esp_ble_gap_cb_param_t_ble_update_duplicate_exceptional_list_cmpl_evt_param,
    ),
    SetChannelsComplete(esp_ble_gap_cb_param_t_ble_set_channels_evt_param),
    /*
    #if (BLE_50_FEATURE_SUPPORT == TRUE)
        READ_PHY_COMPLETE_EVT,
        SET_PREFERED_DEFAULT_PHY_COMPLETE_EVT,
        SET_PREFERED_PHY_COMPLETE_EVT,
        EXT_ADV_SET_RAND_ADDR_COMPLETE_EVT,
        EXT_ADV_SET_PARAMS_COMPLETE_EVT,
        EXT_ADV_DATA_SET_COMPLETE_EVT,
        EXT_SCAN_RSP_DATA_SET_COMPLETE_EVT,
        EXT_ADV_START_COMPLETE_EVT,
        EXT_ADV_STOP_COMPLETE_EVT,
        EXT_ADV_SET_REMOVE_COMPLETE_EVT,
        EXT_ADV_SET_CLEAR_COMPLETE_EVT,
        PERIODIC_ADV_SET_PARAMS_COMPLETE_EVT,
        PERIODIC_ADV_DATA_SET_COMPLETE_EVT,
        PERIODIC_ADV_START_COMPLETE_EVT,
        PERIODIC_ADV_STOP_COMPLETE_EVT,
        PERIODIC_ADV_CREATE_SYNC_COMPLETE_EVT,
        PERIODIC_ADV_SYNC_CANCEL_COMPLETE_EVT,
        PERIODIC_ADV_SYNC_TERMINATE_COMPLETE_EVT,
        PERIODIC_ADV_ADD_DEV_COMPLETE_EVT,
        PERIODIC_ADV_REMOVE_DEV_COMPLETE_EVT,
        PERIODIC_ADV_CLEAR_DEV_COMPLETE_EVT,
        SET_EXT_SCAN_PARAMS_COMPLETE_EVT,
        EXT_SCAN_START_COMPLETE_EVT,
        EXT_SCAN_STOP_COMPLETE_EVT,
        PREFER_EXT_CONN_PARAMS_SET_COMPLETE_EVT,
        PHY_UPDATE_COMPLETE_EVT,
        EXT_ADV_REPORT_EVT,
        SCAN_TIMEOUT_EVT,
        ADV_TERMINATED_EVT,
        SCAN_REQ_RECEIVED_EVT,
        CHANNEL_SELETE_ALGORITHM_EVT,
        PERIODIC_ADV_REPORT_EVT,
        PERIODIC_ADV_SYNC_LOST_EVT,
        PERIODIC_ADV_SYNC_ESTAB_EVT,
    #endif // #if (BLE_50_FEATURE_SUPPORT == TRUE)
        EVT_MAX,
    */
}

impl std::fmt::Debug for GapEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "{}",
            match self {
                GapEvent::AdvertisingDatasetComplete(_) => "AdvertisingDatasetComplete",
                GapEvent::ScanResponseDatasetComplete(_) => "ScanResponseDatasetComplete",
                GapEvent::ScanParameterDatasetComplete(_) => "ScanParameterDatasetComplete",
                GapEvent::ScanResult(_) => "ScanResult",
                GapEvent::RawAdvertisingDatasetComplete(_) => "RawAdvertisingDatasetComplete",
                GapEvent::RawScanResponseDatasetComplete(_) => "RawScanResponseDatasetComplete",
                GapEvent::AdvertisingStartComplete(_) => "AdvertisingStartComplete",
                GapEvent::ScanStartComplete(_) => "ScanStartComplete",
                GapEvent::AuthenticationComplete(_) => "AuthenticationComplete",
                GapEvent::Key(_) => "Key",
                GapEvent::SecurityRequest(_) => "SecurityRequest",
                GapEvent::PasskeyNotification(_) => "PasskeyNotification",
                GapEvent::PasskeyRequest(_) => "PasskeyRequest",
                GapEvent::OOBRequest => "OOBRequest",
                GapEvent::LocalIR => "LocalIR",
                GapEvent::LocalER => "LocalER",
                GapEvent::NumericComparisonRequest => "NumericComparisonRequest",
                GapEvent::AdvertisingStopComplete(_) => "AdvertisingStopComplete",
                GapEvent::ScanStopComplete(_) => "ScanStopComplete",
                GapEvent::SetStaticRandomAddressComplete(_) => "SetStaticRandomAddressComplete",
                GapEvent::UpdateConnectionParamsComplete(_) => "UpdateConnectionParamsComplete",
                GapEvent::SetPacketLengthComplete(_) => "SetPacketLengthComplete",
                GapEvent::SetLocalPrivacy(_) => "SetLocalPrivacy",
                GapEvent::RemoveDeviceBondComplete(_) => "RemoveDeviceBondComplete",
                GapEvent::ClearDeviceBondComplete(_) => "ClearDeviceBondComplete",
                GapEvent::GetDeviceBondComplete(_) => "GetDeviceBondComplete",
                GapEvent::ReadRssiComplete(_) => "ReadRssiComplete",
                GapEvent::UpdateWhitelistComplete(_) => "UpdateWhitelistComplete",
                GapEvent::UpdateDuplicateListComplete(_) => "UpdateDuplicateListComplete",
                GapEvent::SetChannelsComplete(_) => "SetChannelsComplete",
            }
        )
    }
}

impl GapEvent {
    #[allow(non_upper_case_globals)]
    pub(crate) unsafe fn build(
        evt: esp_gap_ble_cb_event_t,
        param: *mut esp_ble_gap_cb_param_t,
    ) -> Self {
        let param = param.as_ref().unwrap();
        match evt {
            esp_gap_ble_cb_event_t_ESP_GAP_BLE_ADV_DATA_SET_COMPLETE_EVT => {
                GapEvent::AdvertisingDatasetComplete(param.adv_data_cmpl)
            }
            esp_gap_ble_cb_event_t_ESP_GAP_BLE_SCAN_RSP_DATA_SET_COMPLETE_EVT => {
                GapEvent::ScanResponseDatasetComplete(param.scan_rsp_data_cmpl)
            }
            esp_gap_ble_cb_event_t_ESP_GAP_BLE_SCAN_PARAM_SET_COMPLETE_EVT => {
                GapEvent::ScanParameterDatasetComplete(param.scan_param_cmpl)
            }
            esp_gap_ble_cb_event_t_ESP_GAP_BLE_SCAN_RESULT_EVT => {
                GapEvent::ScanResult(param.scan_rst)
            }
            esp_gap_ble_cb_event_t_ESP_GAP_BLE_ADV_DATA_RAW_SET_COMPLETE_EVT => {
                GapEvent::RawAdvertisingDatasetComplete(param.adv_data_raw_cmpl)
            }
            esp_gap_ble_cb_event_t_ESP_GAP_BLE_SCAN_RSP_DATA_RAW_SET_COMPLETE_EVT => {
                GapEvent::RawScanResponseDatasetComplete(param.scan_rsp_data_raw_cmpl)
            }
            esp_gap_ble_cb_event_t_ESP_GAP_BLE_ADV_START_COMPLETE_EVT => {
                GapEvent::AdvertisingStartComplete(param.adv_start_cmpl)
            }
            esp_gap_ble_cb_event_t_ESP_GAP_BLE_SCAN_START_COMPLETE_EVT => {
                GapEvent::ScanStartComplete(param.scan_start_cmpl)
            }
            esp_gap_ble_cb_event_t_ESP_GAP_BLE_AUTH_CMPL_EVT => {
                GapEvent::AuthenticationComplete(param.ble_security)
            }
            esp_gap_ble_cb_event_t_ESP_GAP_BLE_KEY_EVT => GapEvent::Key(param.ble_security),
            esp_gap_ble_cb_event_t_ESP_GAP_BLE_SEC_REQ_EVT => {
                GapEvent::SecurityRequest(param.ble_security)
            }
            esp_gap_ble_cb_event_t_ESP_GAP_BLE_PASSKEY_NOTIF_EVT => {
                GapEvent::PasskeyNotification(param.ble_security)
            }
            esp_gap_ble_cb_event_t_ESP_GAP_BLE_PASSKEY_REQ_EVT => {
                GapEvent::PasskeyRequest(param.ble_security)
            }
            esp_gap_ble_cb_event_t_ESP_GAP_BLE_OOB_REQ_EVT => GapEvent::OOBRequest,
            esp_gap_ble_cb_event_t_ESP_GAP_BLE_LOCAL_IR_EVT => GapEvent::LocalIR,
            esp_gap_ble_cb_event_t_ESP_GAP_BLE_LOCAL_ER_EVT => GapEvent::LocalER,
            esp_gap_ble_cb_event_t_ESP_GAP_BLE_NC_REQ_EVT => GapEvent::NumericComparisonRequest,
            esp_gap_ble_cb_event_t_ESP_GAP_BLE_ADV_STOP_COMPLETE_EVT => {
                GapEvent::AdvertisingStopComplete(param.adv_stop_cmpl)
            }
            esp_gap_ble_cb_event_t_ESP_GAP_BLE_SCAN_STOP_COMPLETE_EVT => {
                GapEvent::ScanStopComplete(param.scan_stop_cmpl)
            }
            esp_gap_ble_cb_event_t_ESP_GAP_BLE_SET_STATIC_RAND_ADDR_EVT => {
                GapEvent::SetStaticRandomAddressComplete(param.set_rand_addr_cmpl)
            }
            esp_gap_ble_cb_event_t_ESP_GAP_BLE_UPDATE_CONN_PARAMS_EVT => {
                GapEvent::UpdateConnectionParamsComplete(param.update_conn_params)
            }
            esp_gap_ble_cb_event_t_ESP_GAP_BLE_SET_PKT_LENGTH_COMPLETE_EVT => {
                GapEvent::SetPacketLengthComplete(param.pkt_data_lenth_cmpl)
            }
            esp_gap_ble_cb_event_t_ESP_GAP_BLE_SET_LOCAL_PRIVACY_COMPLETE_EVT => {
                GapEvent::SetLocalPrivacy(param.local_privacy_cmpl)
            }
            esp_gap_ble_cb_event_t_ESP_GAP_BLE_REMOVE_BOND_DEV_COMPLETE_EVT => {
                GapEvent::RemoveDeviceBondComplete(param.remove_bond_dev_cmpl)
            }
            esp_gap_ble_cb_event_t_ESP_GAP_BLE_CLEAR_BOND_DEV_COMPLETE_EVT => {
                GapEvent::ClearDeviceBondComplete(param.clear_bond_dev_cmpl)
            }
            esp_gap_ble_cb_event_t_ESP_GAP_BLE_GET_BOND_DEV_COMPLETE_EVT => {
                GapEvent::GetDeviceBondComplete(param.get_bond_dev_cmpl)
            }
            esp_gap_ble_cb_event_t_ESP_GAP_BLE_READ_RSSI_COMPLETE_EVT => {
                GapEvent::ReadRssiComplete(param.read_rssi_cmpl)
            }
            esp_gap_ble_cb_event_t_ESP_GAP_BLE_UPDATE_WHITELIST_COMPLETE_EVT => {
                GapEvent::UpdateWhitelistComplete(param.update_whitelist_cmpl)
            }
            esp_gap_ble_cb_event_t_ESP_GAP_BLE_UPDATE_DUPLICATE_EXCEPTIONAL_LIST_COMPLETE_EVT => {
                GapEvent::UpdateDuplicateListComplete(param.update_duplicate_exceptional_list_cmpl)
            }
            esp_gap_ble_cb_event_t_ESP_GAP_BLE_SET_CHANNELS_EVT => {
                GapEvent::SetChannelsComplete(param.ble_set_channels)
            }
            _ => {
                log::warn!("Unhandled event {:?}", evt);
                panic!("Unhandled event {:?}", evt)
            }
        }
    }
}
