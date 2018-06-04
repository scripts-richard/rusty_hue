extern crate curl;
extern crate serde;

extern crate serde_json;

#[macro_use]
extern crate serde_derive;

use curl::easy::Easy;
use serde_json::Value;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;

pub struct RGB {
    r: u8,
    g: u8,
    b: u8,
}

impl RGB {
    pub fn from_xy(xy: XY) -> RGB {
        let z = 1.0 - xy.x - xy.y;
        let brightness = xy.brightness as f32;
        let x = brightness / xy.y * xy.x;
        let y = brightness / xy.y * z;

        // Convert to RGB using Wide RGB D65 conversion
        let r = x * 1.656492 - brightness * 0.354851 - y * 0.255038;
        let g = -x * 0.707196 + brightness * 1.655397 + y * 0.036152;
        let b = x * 0.051713 - brightness * 0.121364 + y * 1.011530;

        // Apply reverse gamma correction
        let mut rgb = [r, g, b];

        for i in 0..3 {
            if rgb[i] <= 0.0031308 {
                rgb[i] *= 12.92;
            } else {
                rgb[i] = 1.055 * rgb[i].powf(1.0 / 2.4) - 0.055;
            }

            rgb[i] *= 255.0;
        }

        RGB { r: rgb[0] as u8, g: rgb[1] as u8, b: rgb[2] as u8 }
    }
}

pub struct XY {
    pub x: f32,
    pub y: f32,
    pub brightness: u8,
}

impl XY {
    pub fn from_rgb(rgb: RGB) -> XY {
        let mut rgb = [rgb.r as f32, rgb.g as f32, rgb.b as f32];

        // Apply gamma correction
        for i in 0..3 {
            rgb[i] /= 255.0;

            if rgb[i] > 0.04045 {
                rgb[i] = ((rgb[i] + 0.055) / 1.055).powf(2.3);
            } else {
                rgb[i] /= 12.92;
            }
        }

        let r = rgb[0];
        let g = rgb[1];
        let b = rgb[2];

        // Convert RGB to XYZ using Wide RGB D65 conversion
        let x = r * 0.664511 + g * 0.154324 + b * 0.162028;
        let y = r * 0.283881 + g * 0.668433 + b * 0.047685;
        let z = r * 0.000088 + g * 0.072310 + b * 0.986039;

        let brightness = (y * 254.0) as u8;

        XY{ x: x / (x + y + z), y: y / (x + y + z), brightness: brightness }
    }
}

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub struct LightState {
    on: bool,
    bri: u8,
    hue: u32,
    sat: u8,
    effect: String,
    xy: Vec<f32>,
    ct: u32,
    alert: String,
    colormode: String,
    reachable: bool,
}

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub struct Light {
    state: LightState,
    light_type: String,
    name: String,
    modelid: String,
    manufacturername: String,
    uniqueid: String,
    swversion: String,
}

#[derive(Debug)]
pub struct Hue {
    ip: String,
    token: String,
    base_address: String,
    lights: Vec<Light>,
}

impl Hue {
    pub fn new() -> Result<Hue, Box<Error>> {
        let ip = get_hue_ip()?;
        let token = get_token()?;
        let lights = Vec::new();

        let mut base_address = "http://".to_string() + &ip;
        base_address.push_str("/api/");
        base_address += &token;
        base_address.push_str("/lights");

        let hue = Hue { ip: ip, token: token,  base_address: base_address, lights: lights };

        Ok(hue)
    }

    pub fn get_lights(&mut self) -> Result<(), Box<Error>> {
        let mut data = Vec::new();
        let mut handle = Easy::new();

        handle.url(&self.base_address).unwrap();
        {
            let mut transfer = handle.transfer();
            transfer.write_function(|new_data| {
                data.extend_from_slice(new_data);
                Ok(new_data.len())
            }).unwrap();
            transfer.perform().unwrap();
        }

        let json: Value = serde_json::from_slice(&data)?;

        let mut index = 1;

        while json[index.to_string()].is_object() {
            let light = serde_json::to_string(&json[index.to_string()])?.replace("type", "light_type");
            let light: Light = serde_json::from_str(&light)?;
            self.lights.push(light);
            index += 1;
        }

        Ok(())
    }
}

fn get_hue_ip() -> Result<String, Box<Error>> {
    let mut data = Vec::new();
    let mut handle = Easy::new();
    handle.url("https://www.meethue.com/api/nupnp").unwrap();
    {
        let mut transfer = handle.transfer();
        transfer.write_function(|new_data| {
            data.extend_from_slice(new_data);
            Ok(new_data.len())
        }).unwrap();
        transfer.perform().unwrap();
    }

    let json: Value = serde_json::from_slice(&data)?;

    Ok(json[0]["internalipaddress"].to_string().replace("\"", ""))
}

fn get_token() -> Result<(String), Box<Error>> {
    let mut f = File::open("token.txt")?;
    let mut token = String::new();

    f.read_to_string(&mut token)?;
    token.truncate(40);
    Ok(token)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn rgb_to_xy() {
        let rgb  = RGB { r: 100, g: 100 , b: 100 };
        let xy = XY::from_rgb(rgb);

        assert_eq!(xy.x, 0.32272673);
        assert_eq!(xy.y, 0.32902290);
        assert_eq!(xy.brightness, 35);

        let rgb = RGB { r: 100, g: 10 , b: 100 };
        let xy = XY::from_rgb(rgb);

        assert_eq!(xy.x, 0.38354447);
        assert_eq!(xy.y, 0.15998589);
        assert_eq!(xy.brightness, 12);
    }

    #[test]
    fn xy_to_rgb() {
        let xy = XY { x: 0.32272673, y: 0.32902290, brightness: 35 };
        let rgb = RGB::from_xy(xy);

        assert_eq!(rgb.r, 145);
        assert_eq!(rgb.g, 145);
        assert_eq!(rgb.b, 145);
    }

    #[test]
    fn get_ip() {
        let ip = get_hue_ip();
        assert!(ip.is_ok());
    }

    #[test]
    fn make_light_state() {
        let data = r#"{
            "on": false,
            "bri": 254,
            "hue": 14956,
            "sat": 140,
            "effect": "none",
            "xy": [
            0.4571,
            0.4097
            ],
            "ct": 366,
            "alert": "none",
            "colormode": "ct",
            "reachable": true
        }"#;

        let light_state: LightState = serde_json::from_str(data).unwrap();

        assert_eq!(light_state.colormode, "ct");
        assert!(light_state.reachable);
    }

    #[test]
    fn make_light() {
        let data = r#"{
            "state": {
                "on": false,
                "bri": 254,
                "hue": 14956,
                "sat": 140,
                "effect": "none",
                "xy": [
                    0.4571,
                    0.4097
                ],
                "ct": 366,
                "alert": "none",
                "colormode": "ct",
                "reachable": true
            },
            "type": "Extended color light",
            "name": "Sinnerlig",
            "modelid": "LCT003",
            "manufacturername": "Philips",
            "uniqueid": "00:17:88:01:00:f1:01:17-0b",
            "swversion": "5.50.1.19085"
        }"#.replace("type", "light_type");

        let light: Light = serde_json::from_str(&data).unwrap();

        assert_eq!(light.light_type, "Extended color light");
        assert!(light.state.reachable);
    }

    #[test]
    fn make_hue() {
        let hue = Hue::new();
        assert!(hue.is_ok());
    }
}
