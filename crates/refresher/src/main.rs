extern crate robohome_shared;
extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
#[macro_use]
extern crate log;
extern crate pretty_env_logger;

use robohome_shared::{
    data,
    ipc::send,
};

static WEATHER_URI: &str = include_str!("../api_url");

fn main() {
    if ::std::env::var("RUST_LOG").is_err() {
        ::std::env::set_var("RUST_LOG", "info");
    }
    pretty_env_logger::init();
    info!("Getting new sunrise and sunset times");
    let mut res = match reqwest::get(WEATHER_URI) {
        Ok(res) => res,
        Err(e) => {
            error!("Request Failed {}", e);
            send_and_exit(1);
        }
    };
    let res: WunderGroundResponse = match res.json() {
        Ok(j) => j,
        Err(e) => {
            error!("Deserialization failed: {}", e);
            send_and_exit(2);
        }
    };
    let SunPhase { sunrise, sunset } = res.sun_phase;
    info!("new sunrise: {}:{}", sunrise.hour, sunrise.minute);
    info!("new sunset: {}:{}", sunset.hour, sunset.minute);
    match data::update_special_times(sunrise.hour - 1, sunrise.minute,
                                    sunrise.hour, sunrise.minute,
                                    sunset.hour - 1, sunset.minute,
                                    sunset.hour, sunset.minute) {
        Ok(ct) => info!("update {} special times", ct),
        Err(e) => {
            error!("Failed to update special times: {}", e);
            send_and_exit(3);
        }
    }
}

fn send_and_exit(code: i32) -> ! {
    if let Err(e) = send("database", &()) {
        error!("Failed to send db update message {}", e);
    }
    ::std::process::exit(code);
}

#[derive(Deserialize, Debug)]
struct WunderGroundResponse {
    pub sun_phase: SunPhase,
}

#[derive(Deserialize, Debug)]
struct SunPhase {
    #[serde(deserialize_with = "deserialize_time")]
    pub sunrise: ApiTime,
    #[serde(deserialize_with = "deserialize_time")]
    pub sunset: ApiTime,
}

#[derive(Debug)]
struct ApiTime {
    pub hour: i32,
    pub minute: i32,
}

use serde::{Deserializer, Deserialize, de::Error};
use std::collections::HashMap;
fn deserialize_time<'de, D>(d: D) -> Result<ApiTime, D::Error> where D: Deserializer<'de> {
    type Json = HashMap<String, String>;
    let map = Json::deserialize(d)?;
    let hour = map.get("hour").ok_or(serde::de::Error::missing_field("hour"))?;
    let hour: i32 = hour.parse()
                .map_err(|e| Error::custom(format!("Failed to parse i32 {}", e)))?;
    let minute = map.get("minute").ok_or(serde::de::Error::missing_field("minute"))?;
    let minute: i32 = minute.parse()
                .map_err(|e| Error::custom(format!("Failed to parse i32 {}", e)))?;
    Ok(ApiTime {
        hour,
        minute,
    })
}