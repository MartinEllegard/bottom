use anyhow::Result;
use std::process::Command;

use crate::app::filter::Filter;

use super::{TempHarvest, TemperatureType};

/// Returned devices from grabbing lm_sensors data
/// name/adaptor/sensors
struct LmSensorsDevice {
    name: String,
    adapter: String,
    sensors: Vec<LmSensorsSensor>,
}

/// Returned sensors from grabbing lm_sensors data
/// values/names
struct LmSensorsSensor {
    name: String,
    value: f32,
    sensor_type: LmSensorsSensorType,
}

enum LmSensorsSensorType {
    Temp,
    Fan,
    Voltage,
}

fn get_lm_sensor_data() -> Vec<LmSensorsDevice> {
    if cfg!(target_os = "windows") {
        return Vec::<LmSensorsDevice>::new();
    }

    let command = Command::new("sensors").arg("-u").output();
    let output = match command {
        Ok(val) => String::from_utf8(val.stdout).expect("error"),
        Err(_) => "error".to_string(),
    };

    match output == *"error" {
        true => Vec::<LmSensorsDevice>::new(),
        false => parse_lm_sensors_data(output.as_str()),
    }
}

fn parse_lm_sensors_sensor_type(sensor_name: &str) -> LmSensorsSensorType {
    if sensor_name.contains("temp") {
        LmSensorsSensorType::Temp
    } else if sensor_name.contains("fan") {
        LmSensorsSensorType::Fan
    } else {
        LmSensorsSensorType::Voltage
    }
}

fn format_friendly_names(device_name: String, sensor_name: String) -> String {
    let parent_name = match device_name.clone().to_lowercase() {
        x if x.contains("wifi") => "Wifi".to_string(),
        x if x.contains("gpu") => "Gpu".to_string(),
        x if x.contains("nvidia") => "Gpu".to_string(),
        x if x.contains("it86") => "MB".to_string(),
        x if x.contains("k10") => "CPU".to_string(),
        x if x.contains("kraken") => "AIO".to_string(),
        x if x.contains("nvme") => "Nvme".to_string(),
        _ => device_name
            .split('-')
            .next()
            .expect("device name")
            .to_string(),
    };

    format!("{0}: {1}", parent_name, sensor_name)
}

fn parse_lm_sensors_data(data: &str) -> Vec<LmSensorsDevice> {
    let mut devices = Vec::new();
    let mut lines = data.lines();

    while let Some(line) = lines.next() {
        // Look for device name (e.g., "iwlwifi_1-virtual-0")
        if line.contains("-") {
            let device_name = line.to_string();
            let adapter = lines
                .next()
                .unwrap_or("")
                .replace("Adapter: ", "")
                .to_string();

            let mut sensors = Vec::new();
            while let Some(sensor_line) = lines.next() {
                if sensor_line.trim().is_empty() {
                    break; // end of the device section
                }

                // Parse sensor data
                if sensor_line.trim().ends_with(":") {
                    let sensor_name = sensor_line.trim().trim_end_matches(':').to_string();
                    if let Some(value_line) = lines.next() {
                        match value_line.contains("input") {
                            true => {
                                let parts: Vec<&str> =
                                    value_line.trim_start().split_whitespace().collect();
                                if parts.len() == 2 {
                                    let sensor_value: f32 = parts[1].parse().unwrap_or(0.0);
                                    let sensor_type = parse_lm_sensors_sensor_type(parts[0]);
                                    sensors.push(LmSensorsSensor {
                                        name: sensor_name,
                                        value: sensor_value,
                                        sensor_type,
                                    });
                                }
                            }
                            false => {
                                continue;
                            }
                        };
                    }
                }
            }

            devices.push(LmSensorsDevice {
                name: device_name,
                adapter,
                sensors,
            });
        }
    }

    devices
}

pub fn get_temperature_data(
    temp_type: &TemperatureType, filter: &Option<Filter>,
) -> Result<Option<Vec<TempHarvest>>> {
    let mut temperatures: Vec<TempHarvest> = vec![];

    let sensor_data = get_lm_sensor_data();

    sensor_data.iter().for_each(|device| {
        device.sensors.iter().for_each(|sensor| {
            if let LmSensorsSensorType::Temp = sensor.sensor_type {
                if Filter::optional_should_keep(filter, &sensor.name) {
                    temperatures.push(TempHarvest {
                        name: format_friendly_names(device.name.clone(), sensor.name.clone()),
                        temperature: Some(temp_type.convert_temp_unit(sensor.value)),
                    })
                }
            }
        });
    });

    Ok(Some(temperatures))
}
