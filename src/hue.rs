use curl::easy::Easy;
use serde_json::Value;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::collections::HashMap;
use serde_json;


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
    lights: HashMap<String, Light>,
}


impl Hue {
    pub fn new() -> Result<Hue, Box<Error>> {
        let ip = get_hue_ip()?;
        let token = get_token()?;
        let lights = HashMap::new();

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
            self.lights.insert(index.to_string(), light);
            index += 1;
        }

        Ok(())
    }

    pub fn toggle_lights(mut self) -> Result<(), Box<Error>> {
        let mut all_off = true;

        for (_, light) in self.lights.iter_mut() {
            if light.state.on {
                all_off = false;
                break;
            }
        }

        if all_off {
            self.power(all_off)
        } else {
            self.power(all_off)
        }
    }

    pub fn power(mut self, power: bool) -> Result<(), Box<Error>> {
        for (index, light) in self.lights.iter_mut() {
            if light.state.on != power {
                let mut url = self.base_address.clone() + "/" + index + "/state";
                let mut body = String::from("{\"on\":");
                body += &power.to_string();
                body += "}";
                let mut handle = Easy::new();
                handle.url(&url).unwrap();
                //build post request
                //get post request
            }
        }
        Ok(())
    }
}


fn get_hue_ip() -> Result<String, Box<Error>> {
    let mut data = Vec::new();
    let mut handle = Easy::new();
    handle.url("https://www.meethue.com/api/nupnp")?;
    {
        let mut transfer = handle.transfer();
        transfer.write_function(|new_data| {
            data.extend_from_slice(new_data);
            Ok(new_data.len())
        }).unwrap();
        transfer.perform()?;
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
