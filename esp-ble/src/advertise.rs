use esp_idf_sys::*;

#[allow(clippy::upper_case_acronyms)]
#[repr(u16)]
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

pub struct AdvertiseData {
    pub set_scan_rsp: bool,
    pub include_name: bool,
    pub include_txpower: bool,
    pub min_interval: i32,
    pub max_interval: i32,
    pub manufacturer: Option<String>,
    pub service: Option<String>,
    pub service_uuid: Option<String>,
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

impl TryFrom<AdvertiseData> for esp_ble_adv_data_t {
    type Error = EspError;
    fn try_from(config: AdvertiseData) -> Result<Self, Self::Error> {
        let manufacturer_len = config.manufacturer.as_ref().map(|m| m.len()).unwrap_or(0) as u16;
        let manufacturer = match config.manufacturer {
            Some(m) => {
                let m = std::ffi::CString::new(m)
                    .map_err(|_| EspError::from(ESP_ERR_INVALID_ARG).unwrap())?;
                m.as_ptr()
            }
            None => std::ptr::null_mut(),
        };
        let service_data_len = config.service.as_ref().map(|s| s.len()).unwrap_or(0) as u16;
        let service = match config.service {
            Some(s) => {
                let s = std::ffi::CString::new(s)
                    .map_err(|_| EspError::from(ESP_ERR_INVALID_ARG).unwrap())?;
                s.as_ptr()
            }
            None => std::ptr::null_mut(),
        };
        Ok(esp_ble_adv_data_t {
            set_scan_rsp: config.set_scan_rsp,
            include_name: config.include_name,
            include_txpower: config.include_txpower,
            min_interval: config.min_interval,
            max_interval: config.max_interval,
            manufacturer_len,
            p_manufacturer_data: manufacturer as *mut u8,
            service_data_len,
            p_service_data: service as *mut u8,
            service_uuid_len: 0,
            p_service_uuid: std::ptr::null_mut(),
            appearance: config.appearance.into(),
            flag: config.flag,
        })
    }
}
