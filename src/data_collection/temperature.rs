//! Data collection for temperature metrics.
//!
//! For Linux and macOS, this is handled by Heim.
//! For Windows, this is handled by sysinfo.

cfg_if::cfg_if! {
    if #[cfg(feature = "lmsensors")] {
        pub mod lm_sensors;
        pub use self::lm_sensors::*;
    }
    else if #[cfg(target_os = "linux")] {
        pub mod linux;
        pub use self::linux::*;
    } else if #[cfg(any(target_os = "freebsd", target_os = "macos", target_os = "windows", target_os = "android", target_os = "ios"))] {
        pub mod sysinfo;
        pub use self::sysinfo::*;
    }
}

use std::str::FromStr;

#[derive(Default, Debug, Clone)]
pub struct TempHarvest {
    pub name: String,
    pub temperature: Option<f32>,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Default)]
pub enum TemperatureType {
    #[default]
    Celsius,
    Kelvin,
    Fahrenheit,
}

impl FromStr for TemperatureType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "fahrenheit" | "f" => Ok(TemperatureType::Fahrenheit),
            "kelvin" | "k" => Ok(TemperatureType::Kelvin),
            "celsius" | "c" => Ok(TemperatureType::Celsius),
            _ => Err(format!(
                "'{s}' is an invalid temperature type, use one of: [kelvin, k, celsius, c, fahrenheit, f]."
            )),
        }
    }
}

impl TemperatureType {
    /// Given a temperature in Celsius, covert it if necessary for a different
    /// unit.
    pub fn convert_temp_unit(&self, temp_celsius: f32) -> f32 {
        fn convert_celsius_to_kelvin(celsius: f32) -> f32 {
            celsius + 273.15
        }

        fn convert_celsius_to_fahrenheit(celsius: f32) -> f32 {
            (celsius * (9.0 / 5.0)) + 32.0
        }

        match self {
            TemperatureType::Celsius => temp_celsius,
            TemperatureType::Kelvin => convert_celsius_to_kelvin(temp_celsius),
            TemperatureType::Fahrenheit => convert_celsius_to_fahrenheit(temp_celsius),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::data_collection::temperature::TemperatureType;

    #[test]
    fn temp_conversions() {
        const TEMP: f32 = 100.0;

        assert_eq!(
            TemperatureType::Celsius.convert_temp_unit(TEMP),
            TEMP,
            "celsius to celsius is the same"
        );

        assert_eq!(TemperatureType::Kelvin.convert_temp_unit(TEMP), 373.15);

        assert_eq!(TemperatureType::Fahrenheit.convert_temp_unit(TEMP), 212.0);
    }
}
