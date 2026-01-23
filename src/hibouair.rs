use hex;
use zerocopy::{FromBytes, Unaligned, Immutable, KnownLayout};

pub enum VocType {
    Old = 0,
    Resistance = 1,
    Ppm = 2,
    Iaq = 3,
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum SensorType {
    Unknown = 0x00,
    TEMP_HUM_SENSOR = 0x02,
    PM_SENSOR = 0x03,
    CO2_SENSOR = 0x04,
    NO2_OUTDOOR_WIFI = 0x05,
    CO2_BATTERY = 0x06,
    NO2_OUTDOOR_LTEM_NBIOT = 0x07,
    PIR_SENSOR = 0x08,
    CO2_NOISE = 0x09,
    DUO_MASTER = 0x0A,
    DUO_SLAVE = 0x0B,
    MATRIX = 0x14,
}

impl TryFrom<i64> for SensorType {
    type Error = &'static str;

    fn try_from(value: i64) -> std::result::Result<Self, Self::Error> {
        match value {
            0x00 => Ok(SensorType::Unknown),
            0x02 => Ok(SensorType::TEMP_HUM_SENSOR),
            0x03 => Ok(SensorType::PM_SENSOR),
            0x04 => Ok(SensorType::CO2_SENSOR),
            0x05 => Ok(SensorType::NO2_OUTDOOR_WIFI),
            0x06 => Ok(SensorType::CO2_BATTERY),
            0x07 => Ok(SensorType::NO2_OUTDOOR_LTEM_NBIOT),
            0x08 => Ok(SensorType::PIR_SENSOR),
            0x09 => Ok(SensorType::CO2_NOISE),
            0x0A => Ok(SensorType::DUO_MASTER),
            0x0B => Ok(SensorType::DUO_SLAVE),
            0x14 => Ok(SensorType::MATRIX),
            _ => Err("Unkown sensor type"),
        }
    }
}

// #[derive(Debug, Clone, PartialEq, Copy)]
#[repr(C, packed)]
#[derive(FromBytes, Unaligned, Immutable, KnownLayout, Debug, Clone, Copy, PartialEq)]
pub struct HibouAir {
    mfid: u16,          // the manufacturer id of the device
    beacon_nr: u8,      // type of beacon
    board_type: u8,     // type of device
    board_id: [u8;3],   // unique board id
    als: u16,           // ambient light sensor
    bar: u16,           // pressure
    temp: u16,          // temperature
    hum: u16,           // humidity
    voc: u16,           // volatile organic compounds
    pm1_0: u16,         // particle matter PM1.0
    pm2_5: u16,         // particle matter PM2.5
    pm10: u16,          // particle matter PM10.0
    co2: u16,           // carbon dioxide
    voc_type: u8,       // 0 = old, 1 = resistance, 2 = ppm, 3 = IAQ
}
// 0201061BFF5B07050422005A0000BA27C60017013E0000000000000001C002

impl HibouAir {
    /// Tar en hex-sträng och försöker konvertera den till en HibouAir-struct
    pub fn from_hex(hex_str: &str) -> std::result::Result<Self, String> {
        // 1. Konvertera hex till bytes
        let bytes = hex::decode(hex_str)
            .map_err(|e| format!("Ogiltig hex-sträng: {}", e))?;

        // 2. Försök läsa structen från början av byten
        // read_from_prefix returnerar Result<(Self, &[u8]), CastError>
        let (data, _rest) = Self::read_from_prefix(&bytes[5..])
            .map_err(|_| "Datan är för kort för att matcha HibouAir-formatet")?;

        // 3. Returnera den kopierade structen
        Ok(data)
    }

    // Return a string representation of the HibouAir struct.
    fn to_string(&self) -> String {
        format!(
            "HibouAir(mfid: {}, beacon_nr: {}, board_type: {}, board_id: {:02X?}, als: {}, bar: {}, temp: {}, hum: {}, voc: {}, pm1_0: {}, pm2_5: {}, pm10: {}, co2: {}, voc_type: {})",
            {self.mfid},
            self.beacon_nr,
            self.board_type,
            self.board_id,
            {self.als},
            {self.bar},
            {self.temp},
            {self.hum},
            {self.voc},
            {self.pm1_0},
            {self.pm2_5},
            {self.pm10},
            {self.co2},
            self.voc_type
        )
    }

    // Getter methods for each field.

    // Return board ID as u32.
    pub fn get_id(&self) -> u32 {
        ((self.board_id[0] as u32) << 16) | ((self.board_id[1] as u32) << 8) | (self.board_id[2] as u32)
    }

    // Return board ID as hex string.
    pub fn get_board_id_string(&self) -> String {
        format!("{:02X}", self.get_id())
    }

    // Return board type as SensorType.
    pub fn get_board_type(&self) -> SensorType {
        SensorType::try_from(self.board_type as i64).unwrap_or(SensorType::Unknown) 
    }

    // Return board type as string.
    pub fn get_board_type_string(&self) -> String {
        match self.get_board_type() {
            SensorType::PM_SENSOR => "PM".to_string(),
            SensorType::CO2_SENSOR => "CO2".to_string(),
            SensorType::TEMP_HUM_SENSOR => "Temp/Hum".to_string(),
            SensorType::NO2_OUTDOOR_WIFI => "NO2 Outdoor WiFi".to_string(),
            SensorType::CO2_BATTERY => "CO2 Battery".to_string(),
            SensorType::NO2_OUTDOOR_LTEM_NBIOT => "NO2 Outdoor LTEM NBIOT".to_string(),
            SensorType::PIR_SENSOR => "PIR".to_string(),
            SensorType::CO2_NOISE => "CO2 Noise".to_string(),
            SensorType::DUO_MASTER => "Duo Master".to_string(),
            SensorType::DUO_SLAVE => "Duo Slave".to_string(),
            SensorType::MATRIX => "Matrix".to_string(),
            _ => "Unknown".to_string(),
        }
    }

    // Return ambient light sensor value.
    pub fn get_als(&self) -> u16 {
        self.als
    }

    // Return barometric pressure value.
    pub fn get_bar(&self) -> f64 {
        self.bar as f64 / 10.0
    }

    // Return temperature value.
    pub fn get_temp(&self) -> f64 {
        (self.temp as i16) as f64 / 10.0
    }

    // Return humidity value.
    pub fn get_hum(&self) -> f64 {
        self.hum as f64 / 10.0
    }

    // Return CO2 value.
    pub fn get_co2(&self) -> u16 {
        self.co2.swap_bytes()
    }

    // Return VOC value.
    pub fn get_voc(&self) -> f64 {
        let mut v: f64 = self.voc as f64 ;
        if self.voc_type == 2 {
            v = v / 100.0;
        }
        v
    }

    // Return VOC type.
    pub fn get_voc_type(&self) -> u8 {
        self.voc_type
    }

    // Return VOC unit as string.
    pub fn get_voc_unit(&self) -> String {
        // println!("Voc type: {}", self.voc_type);
        match self.voc_type {
            0 => "".to_string(),
            1 => "".to_string(),
            2 => "ppm".to_string(),
            3 => "IAQ".to_string(),
            _ => "".to_string(),
        }
    }

    // Return VOC value with unit as string.
    pub fn get_voc_view(&self) -> String {
        match self.voc_type {
            0 => "".to_string(),
            1 => "".to_string(),
            2 => format!("{:.1} {}",self.get_voc(), self.get_voc_unit()),
            3 => format!("{:.1} {}",self.get_voc(), self.get_voc_unit()),
            _ => "".to_string(),
        }
    }

    // Return PM1.0 value.
    pub fn get_pm1_0(&self) -> f64 {
        self.pm1_0 as f64 / 10.0
    }

    // Return PM2.5 value.
    pub fn get_pm2_5(&self) -> f64 {
        self.pm2_5 as f64 / 10.0
    }

    // Return PM10 value.
    pub fn get_pm10(&self) -> f64 {
        self.pm10 as f64 / 10.0
    }
}

