use esp_idf_sys::*;

#[derive(Debug, Clone)]
pub enum BtUuid {
    Uuid16(u16),
    Uuid32(u32),
    Uuid128([u8; 16]),
}

impl From<BtUuid> for esp_bt_uuid_t {
    fn from(svc: BtUuid) -> Self {
        let mut bt_uuid: esp_bt_uuid_t = Default::default();
        match svc {
            BtUuid::Uuid16(uuid) => {
                bt_uuid.len = 2;
                bt_uuid.uuid.uuid16 = uuid;
            }
            BtUuid::Uuid32(uuid) => {
                bt_uuid.len = 4;
                bt_uuid.uuid.uuid32 = uuid;
            }
            BtUuid::Uuid128(uuid) => {
                bt_uuid.len = 16;
                bt_uuid.uuid.uuid128 = uuid;
            }
        }
        bt_uuid
    }
}