use crate::BtUuid;
use esp_idf_sys::*;

#[allow(clippy::upper_case_acronyms)]
#[repr(u16)]
#[derive(Clone, Copy, Debug)]
pub enum AppearanceCategory {
    Unknown = 0x00,
    Phone,
    Computer,
    Watch,
    Clock,
    Display,
    RemoteControl,
    EyeGlass,
    Tag,
    Keyring,
    MediaPlayer,
    BarcodeScanner,
    Thermometer,
    HeartRateSensor,
    BloodPressure,
    HumanInterfaceDevice,
    GlucoseMeter,
    RunningWalkingSensor,
    Cycling,
    ControlDevice,
    NetworkDevice,
    Sensor,
    LightFixtures,
    Fan,
    HVAC,
    AirConditionning,
    Humidifier,
    Heating,
    AccessControl,
    MotorizedDevice,
    PowerDevice,
    LightSource,
    WindowCovering,
    AudioSink,
    AudioSource,
    MotorizedVehicle,
    DomesticAppliance,
    WearableAudioDevice,
    Aircraft,
    AVEquipment,
    DisplayEquipment,
    HearingAid,
    Gaming,
    Signage,
    PulseOximeter = 0x31,
    WeightScale,
    PersonalMobilityDevice,
    ContinuousGlucoseMonitor,
    InsulinPump,
    MedicationDelivery,
    OutdoorSportsActivity = 0x51,
}

impl From<AppearanceCategory> for i32 {
    fn from(cat: AppearanceCategory) -> Self {
        ((cat as u16) << 6) as _
    }
}

pub struct RawAdvertiseData {
    data: Vec<AdvertiseType>,
    pub(crate) set_scan_rsp: bool,
}

impl RawAdvertiseData {
    pub fn new(data: Vec<AdvertiseType>, set_scan_rsp: bool) -> Self {
        RawAdvertiseData { data, set_scan_rsp }
    }

    pub fn as_raw_data(&self) -> (*mut u8, u32) {
        let mut v: Vec<u8> = self
            .data
            .iter()
            .flat_map(|v: &AdvertiseType| {
                let v: Vec<u8> = v.into();
                v
            })
            .collect();
        v.shrink_to(31);
        let v = v.as_mut_slice();
        log::info!("Adv data({}): {{ {:?} }}", v.len(), &v);
        (v.as_mut_ptr(), v.len() as u32)
    }
}

#[derive(Clone, Debug)]
pub enum AdvertiseType {
    Flags(u8),
    ServicePartial16(Vec<u16>),
    ServiceComplete16(Vec<u16>),
    ServicePartial32(Vec<u32>),
    ServiceComplete32(Vec<u32>),
    ServicePartial128(Vec<[u8; 16]>),
    ServiceComplete128(Vec<[u8; 16]>),
    IntervalRange(u16, u16),
    DeviceNameShort(String),
    DeviceNameComplete(String),
    Appearance(AppearanceCategory),
    TxPower(u8),
}

impl From<&AdvertiseType> for Vec<u8> {
    fn from(adv: &AdvertiseType) -> Self {
        match adv {
            AdvertiseType::Flags(flag) => vec![0x02, 0x01, *flag],
            AdvertiseType::ServicePartial16(svc) => {
                let l = svc.len() * 2 + 1;
                let mut v = vec![l as u8, 0x02];
                v.append(&mut svc.iter().flat_map(|svc| svc.to_be_bytes()).collect());
                v
            }
            AdvertiseType::ServiceComplete16(svc) => {
                let l = svc.len() * 2 + 1;
                let mut v = vec![l as u8, 0x03];
                v.append(&mut svc.iter().flat_map(|svc| svc.to_be_bytes()).collect());
                v
            }
            AdvertiseType::ServicePartial32(svc) => {
                let l = svc.len() * 4 + 1;
                let mut v = vec![l as u8, 0x04];
                v.append(&mut svc.iter().flat_map(|svc| svc.to_be_bytes()).collect());
                v
            }
            AdvertiseType::ServiceComplete32(svc) => {
                let l = svc.len() * 4 + 1;
                let mut v = vec![l as u8, 0x05];
                v.append(&mut svc.iter().flat_map(|svc| svc.to_be_bytes()).collect());
                v
            }
            AdvertiseType::ServicePartial128(svc) => {
                let l = svc.len() * 16 + 1;
                let mut v = vec![l as u8, 0x06];
                v.append(&mut svc.iter().flat_map(|svc| *svc).collect());
                v
            }
            AdvertiseType::ServiceComplete128(svc) => {
                let l = svc.len() * 16 + 1;
                let mut v = vec![l as u8, 0x07];
                v.append(&mut svc.iter().flat_map(|svc| *svc).collect());
                v
            }
            AdvertiseType::IntervalRange(min, max) => {
                let mut v = vec![0x06, 0x12];
                v.append(&mut min.to_be_bytes().to_vec());
                v.append(&mut max.to_be_bytes().to_vec());
                v.append(&mut vec![0x00]);
                v
            }
            AdvertiseType::DeviceNameShort(name) => {
                let mut v = vec![(name.len() + 1) as u8, 0x08];
                v.append(&mut name.as_bytes().to_vec());
                v
            }
            AdvertiseType::DeviceNameComplete(name) => {
                let mut v = vec![(name.len() + 1) as u8, 0x09];
                v.append(&mut name.as_bytes().to_vec());
                v
            }
            AdvertiseType::Appearance(cat) => {
                let cat: i32 = cat.clone().into();
                vec![0x02, 0x19, cat as u8]
            }
            AdvertiseType::TxPower(_pow) => {
                vec![0x03, 0x0a, 0x09]
            }
        }
    }
}

pub struct AdvertiseData {
    pub set_scan_rsp: bool,
    pub include_name: bool,
    pub include_txpower: bool,
    pub min_interval: i32,
    pub max_interval: i32,
    pub manufacturer: Option<String>,
    pub service: Option<String>,
    pub service_uuid: Option<BtUuid>,
    pub appearance: AppearanceCategory,
    pub flag: u8,
}

impl Default for AdvertiseData {
    fn default() -> Self {
        Self {
            set_scan_rsp: false,
            include_name: false,
            include_txpower: false,
            min_interval: 0,
            max_interval: 0,
            manufacturer: None,
            service: None,
            service_uuid: None,
            appearance: AppearanceCategory::Unknown,
            flag: ESP_BLE_ADV_FLAG_NON_LIMIT_DISC as _,
        }
    }
}

