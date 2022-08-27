use esp_idf_sys::*;

use crate::BtUuid;

enum GattApplicationStatus {
    Unregistered,
    Registered(esp_gatt_if_t),
}

pub struct GattApplication {
    id: u16,
    status: GattApplicationStatus,
}

#[derive(Debug)]
pub struct GattService {
    pub(crate) is_primary: bool,
    pub(crate) id: BtUuid,
    pub(crate) instance_id: u8,
    pub(crate) handle: u16,
}

impl GattService {
    pub fn new_primary(id: BtUuid, handle: u16, instance_id: u8) -> Self {
        Self {
            is_primary: true,
            id,
            handle,
            instance_id,
        }
    }

    pub fn new(id: BtUuid, handle: u16, instance_id: u8) -> Self {
        Self {
            is_primary: false,
            id,
            handle,
            instance_id,
        }
    }
}

impl GattApplication {
    pub fn new(id: u16) -> Self {
        Self {
            id,
            status: GattApplicationStatus::Unregistered,
        }
    }

    pub fn get_id(&self) -> u16 {
        self.id
    }

    pub(crate) fn register(&mut self, gatt_if: esp_gatt_if_t) {
        self.status = GattApplicationStatus::Registered(gatt_if);
    }

    pub fn get_gatt_if(&self) -> Result<esp_gatt_if_t, EspError> {
        match self.status {
            GattApplicationStatus::Unregistered => {
                Err(EspError::from(ESP_ERR_INVALID_STATE).unwrap())
            }
            GattApplicationStatus::Registered(gatt_if) => Ok(gatt_if),
        }
    }
}

#[derive(Copy, Clone)]
pub enum GattServiceEvent {
    Register(esp_ble_gatts_cb_param_t_gatts_reg_evt_param),
    Read(esp_ble_gatts_cb_param_t_gatts_read_evt_param),
    Write(esp_ble_gatts_cb_param_t_gatts_write_evt_param),
    ExecWrite(esp_ble_gatts_cb_param_t_gatts_exec_write_evt_param),
    Mtu(esp_ble_gatts_cb_param_t_gatts_mtu_evt_param),
    Confirm(esp_ble_gatts_cb_param_t_gatts_conf_evt_param),
    Unregister(esp_ble_gatts_cb_param_t_gatts_create_evt_param),
    Create(esp_ble_gatts_cb_param_t_gatts_create_evt_param),
    AddIncludedServiceComplete(esp_ble_gatts_cb_param_t_gatts_add_incl_srvc_evt_param),
    AddCharacteristicComplete(esp_ble_gatts_cb_param_t_gatts_add_char_evt_param),
    AddDescriptorComplete(esp_ble_gatts_cb_param_t_gatts_add_char_descr_evt_param),
    DeleteComplete(esp_ble_gatts_cb_param_t_gatts_delete_evt_param),
    StartComplete(esp_ble_gatts_cb_param_t_gatts_start_evt_param),
    StopComplete(esp_ble_gatts_cb_param_t_gatts_stop_evt_param),
    Connect(esp_ble_gatts_cb_param_t_gatts_connect_evt_param),
    Disconnect(esp_ble_gatts_cb_param_t_gatts_disconnect_evt_param),
    Open(esp_ble_gatts_cb_param_t_gatts_open_evt_param),
    Close(esp_ble_gatts_cb_param_t_gatts_close_evt_param),
    Listen(esp_ble_gatts_cb_param_t_gatts_congest_evt_param),
    Congest(esp_ble_gatts_cb_param_t_gatts_congest_evt_param),
    ResponseComplete(esp_ble_gatts_cb_param_t_gatts_rsp_evt_param),
    CreateAttributeTableComplete(esp_ble_gatts_cb_param_t_gatts_add_attr_tab_evt_param),
    SetAttributeValueComplete(esp_ble_gatts_cb_param_t_gatts_set_attr_val_evt_param),
    SendServiceChangeComplete(esp_ble_gatts_cb_param_t_gatts_send_service_change_evt_param),
}

impl std::fmt::Debug for GattServiceEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            GattServiceEvent::Register(reg) => write!(
                f,
                "Register {{ status: {}, app_id: {} }}",
                reg.status, reg.app_id
            ),
            GattServiceEvent::Read(read) => write!(f, "Read {{ {:?} }}", read),
            GattServiceEvent::Write(write) => write!(f, "Write {{ {:?} }}", write),
            GattServiceEvent::ExecWrite(_) => write!(f, "ExecWrite"),
            GattServiceEvent::Mtu(_) => write!(f, "Mtu"),
            GattServiceEvent::Confirm(_) => write!(f, "Confirm"),
            GattServiceEvent::Unregister(_) => write!(f, "Unregister"),
            GattServiceEvent::Create(_) => write!(f, "Create"),
            GattServiceEvent::AddIncludedServiceComplete(_) => {
                write!(f, "AddIncludedServiceComplete")
            }
            GattServiceEvent::AddCharacteristicComplete(_) => {
                write!(f, "AddCharacteristicComplete")
            }
            GattServiceEvent::AddDescriptorComplete(_) => write!(f, "AddDescriptorComplete"),
            GattServiceEvent::DeleteComplete(_) => write!(f, "DeleteComplete"),
            GattServiceEvent::StartComplete(_) => write!(f, "StartComplete"),
            GattServiceEvent::StopComplete(_) => write!(f, "StopComplete"),
            GattServiceEvent::Connect(_) => write!(f, "Connect"),
            GattServiceEvent::Disconnect(_) => write!(f, "Disconnect"),
            GattServiceEvent::Open(_) => write!(f, "Open"),
            GattServiceEvent::Close(_) => write!(f, "Close"),
            GattServiceEvent::Listen(_) => write!(f, "Listen"),
            GattServiceEvent::Congest(_) => write!(f, "Congest"),
            GattServiceEvent::ResponseComplete(_) => write!(f, "ResponseComplete"),
            GattServiceEvent::CreateAttributeTableComplete(_) => {
                write!(f, "CreateAttributeTableComplete")
            }
            GattServiceEvent::SetAttributeValueComplete(_) => {
                write!(f, "SetAttributeValueComplete")
            }
            GattServiceEvent::SendServiceChangeComplete(_) => {
                write!(f, "SendServiceChangeComplete")
            }
        }
    }
}

impl GattServiceEvent {
    pub(crate) unsafe fn build(
        event: esp_idf_sys::esp_gatts_cb_event_t,
        param: *mut esp_idf_sys::esp_ble_gatts_cb_param_t,
    ) -> Self {
        let param: &esp_idf_sys::esp_ble_gatts_cb_param_t = param.as_ref().unwrap();
        match event {
            esp_idf_sys::esp_gatts_cb_event_t_ESP_GATTS_REG_EVT => {
                GattServiceEvent::Register(param.reg)
            }
            esp_idf_sys::esp_gatts_cb_event_t_ESP_GATTS_READ_EVT => {
                GattServiceEvent::Read(param.read)
            }
            esp_idf_sys::esp_gatts_cb_event_t_ESP_GATTS_WRITE_EVT => {
                GattServiceEvent::Write(param.write)
            }
            esp_idf_sys::esp_gatts_cb_event_t_ESP_GATTS_EXEC_WRITE_EVT => {
                GattServiceEvent::ExecWrite(param.exec_write)
            }
            esp_idf_sys::esp_gatts_cb_event_t_ESP_GATTS_MTU_EVT => GattServiceEvent::Mtu(param.mtu),
            esp_idf_sys::esp_gatts_cb_event_t_ESP_GATTS_CONF_EVT => {
                GattServiceEvent::Confirm(param.conf)
            }
            esp_idf_sys::esp_gatts_cb_event_t_ESP_GATTS_UNREG_EVT => {
                GattServiceEvent::Unregister(param.create)
            }
            esp_idf_sys::esp_gatts_cb_event_t_ESP_GATTS_CREATE_EVT => {
                GattServiceEvent::Create(param.create)
            }
            esp_idf_sys::esp_gatts_cb_event_t_ESP_GATTS_ADD_INCL_SRVC_EVT => {
                GattServiceEvent::AddIncludedServiceComplete(param.add_incl_srvc)
            }
            esp_idf_sys::esp_gatts_cb_event_t_ESP_GATTS_ADD_CHAR_EVT => {
                GattServiceEvent::AddCharacteristicComplete(param.add_char)
            }
            esp_idf_sys::esp_gatts_cb_event_t_ESP_GATTS_ADD_CHAR_DESCR_EVT => {
                GattServiceEvent::AddDescriptorComplete(param.add_char_descr)
            }
            esp_idf_sys::esp_gatts_cb_event_t_ESP_GATTS_DELETE_EVT => {
                GattServiceEvent::DeleteComplete(param.del)
            }
            esp_idf_sys::esp_gatts_cb_event_t_ESP_GATTS_START_EVT => {
                GattServiceEvent::StartComplete(param.start)
            }
            esp_idf_sys::esp_gatts_cb_event_t_ESP_GATTS_STOP_EVT => {
                GattServiceEvent::StopComplete(param.stop)
            }
            esp_idf_sys::esp_gatts_cb_event_t_ESP_GATTS_CONNECT_EVT => {
                GattServiceEvent::Connect(param.connect)
            }
            esp_idf_sys::esp_gatts_cb_event_t_ESP_GATTS_DISCONNECT_EVT => {
                GattServiceEvent::Disconnect(param.disconnect)
            }
            esp_idf_sys::esp_gatts_cb_event_t_ESP_GATTS_OPEN_EVT => {
                GattServiceEvent::Open(param.open)
            }
            esp_idf_sys::esp_gatts_cb_event_t_ESP_GATTS_CLOSE_EVT => {
                GattServiceEvent::Close(param.close)
            }
            esp_idf_sys::esp_gatts_cb_event_t_ESP_GATTS_LISTEN_EVT => {
                GattServiceEvent::Listen(param.congest)
            }
            esp_idf_sys::esp_gatts_cb_event_t_ESP_GATTS_CONGEST_EVT => {
                GattServiceEvent::Congest(param.congest)
            }
            esp_idf_sys::esp_gatts_cb_event_t_ESP_GATTS_RESPONSE_EVT => {
                GattServiceEvent::ResponseComplete(param.rsp)
            }
            esp_idf_sys::esp_gatts_cb_event_t_ESP_GATTS_CREAT_ATTR_TAB_EVT => {
                GattServiceEvent::CreateAttributeTableComplete(param.add_attr_tab)
            }
            esp_idf_sys::esp_gatts_cb_event_t_ESP_GATTS_SET_ATTR_VAL_EVT => {
                GattServiceEvent::SetAttributeValueComplete(param.set_attr_val)
            }
            esp_idf_sys::esp_gatts_cb_event_t_ESP_GATTS_SEND_SERVICE_CHANGE_EVT => {
                GattServiceEvent::SendServiceChangeComplete(param.service_change)
            }
            _ => {
                log::warn!("Unhandled event: {:?}", event);
                panic!("Unhandled event: {:?}", event)
            }
        }
    }
}
