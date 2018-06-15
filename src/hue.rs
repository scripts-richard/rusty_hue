//! # hue
//!
//! Collection of data structures, functions, and methods for iteracting with Philips Hue lights.

use reqwest;
use serde_json;
use serde_json::Value;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;

use colors;

/// Represents the state field of a light. Matches the JSON data fields to allow for serialization.
#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub struct LightState {
    on: bool,
    bri: u8,
    hue: u16,
    sat: u8,
    effect: String,
    xy: Vec<f32>,
    ct: u32,
    alert: String,
    colormode: String,
    reachable: bool,
}

/// Represents a single light. Matches the JSON data fields to allow for serialization. Note: type
/// is a rust keyword and must be changed to light_type before use of the data structure.
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

/// Represents a Hue system.
#[derive(Debug)]
pub struct Hue {
    ip: String,
    token: String,
    base_address: String,
    lights: HashMap<String, Light>,
}


impl Hue {
    /// Finds the IP and lights of a Hue system and returns them in a Hue data structure. Requires
    /// an API token.
    pub fn new() -> Result<Hue, Box<Error>> {
        let ip = get_hue_ip()?;
        let token = get_token()?;
        let lights = HashMap::new();

        let base_address = format!("http://{}/api/{}/lights", ip, token);

        let mut hue = Hue { ip: ip,
                            token: token,
                            base_address: base_address,
                            lights: lights };

        hue.get_lights()?;

        Ok(hue)
    }

    /// Helper function to get the Hue lights, deserialize them into data structures, and add them
    /// to a Hue data structure.
    fn get_lights(&mut self) -> Result<(), Box<Error>> {
        let body = reqwest::get(&self.base_address)?.text()?;
        let json: Value = serde_json::from_str(&body)?;

        let mut index = 1;

        while json[index.to_string()].is_object() {
            let light = serde_json::to_string(&json[index.to_string()])?.replace("type", "light_type");
            let light: Light = serde_json::from_str(&light)?;
            self.lights.insert(index.to_string(), light);
            index += 1;
        }
        Ok(())
    }

    /// Helper function for setting all lights to the same power state.
    fn power(&self, power: bool) -> Result<(), Box<Error>> {
        for (index, light) in &self.lights {
            if light.state.reachable && light.state.on != power {
                let body = format!("{{\"on\":{}}}", power);
                let client = reqwest::Client::new();
                let url = format!("{}/{}/state", self.base_address, index);

                client.put(&url).body(body).send()?;
            }
        }
        Ok(())
    }

    /// Helper function for setting the color value by RGB for a single light given its index.
    fn set_color_by_index_and_rgb(&self, index: &str, rgb: &colors::RGB) -> Result<(), Box<Error>> {
        if !self.lights.contains_key(index) {
            return Err(From::from(format!("Light index '{}' does not exist.", index)));
        }

        let mut xy = colors::XY::from_rgb(rgb);

        match colors::color_gamut_lookup(self.lights[index].modelid.as_ref()) {
            Some('A') => xy.adjust_for_gamut(colors::COLOR_GAMUT_A),
            Some('B') => xy.adjust_for_gamut(colors::COLOR_GAMUT_B),
            Some('C') => xy.adjust_for_gamut(colors::COLOR_GAMUT_C),
            Some(_) | None => ()
        }


        let url = format!("{}/{}/state", self.base_address, index);
        let body = format!("{{\"bri\": {}, \"xy\": {} }}", xy.brightness, xy.xy_string());

        let client = reqwest::Client::new();
        client.put(&url).body(body).send()?;

        Ok(())
    }

    /// Toggles all lights such that they have the same power state. If one light is on, will turn
    /// it off. If all lights aer off, will turn them all on.
    pub fn toggle_lights(&self) -> Result<(), Box<Error>> {
        let mut all_off = true;

        for (_, light) in &self.lights {
            if light.state.reachable && light.state.on {
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

    /// Prints all fields of a Light and LightState structure in an easily readble format.
    pub fn print_info(&self) {
        for (index, light) in &self.lights {
            println!("Light {}:", index);
            println!("\tName: {}", light.name);
            println!("\tType: {}", light.light_type);
            println!("\tModel ID: {}", light.modelid);
            println!("\tManufacturer: {}", light.manufacturername);
            println!("\tUnique ID: {}", light.uniqueid);
            println!("\tSoftware Version: {}", light.swversion);
            println!("\tState:");
            println!("\t\tOn: {}", light.state.on);
            println!("\t\tBrightness: {}", light.state.bri);
            println!("\t\tHue: {}", light.state.hue);
            println!("\t\tSaturation: {}", light.state.sat);
            println!("\t\tEffect: {}", light.state.effect);
            println!("\t\tx: {}\ty: {}", light.state.xy[0], light.state.xy[1]);
            println!("\t\tColor Temperature: {}", light.state.ct);
            println!("\t\tAlert: {}", light.state.alert);
            println!("\t\tColor Mode: {}", light.state.colormode);
            println!("\t\tReachable: {}", light.state.reachable);
        }
    }

    /// Given the index of a light and RGB color, will set the color of that light.
    pub fn set_color_by_index_and_color(&self, index: &str, color: &str) -> Result<(), Box<Error>> {
        if !self.lights.contains_key(index) {
            return Err(From::from(format!("Light index '{}' does not exist.", index)));
        }

        let colors = colors::load_colors_from_file()?;
        if !colors.contains_key(color) {
            return Err(From::from(format!("Color value '{}' not set.", color)));
        }

        let rgb = &colors[color];

        self.set_color_by_index_and_rgb(index, rgb)?;

        Ok(())
    }

    /// Given the name of a light and RGB color, will set the color of that light.
    pub fn set_color_by_name_and_color(&self, name: &str, color: &str) -> Result<(), Box<Error>> {
        let mut found = false;

        for (index, light) in &self.lights {
            if light.name == name {
                self.set_color_by_index_and_color(index, color)?;
                found = true;
            }
        }

        if !found {
            return Err(From::from(format!("No light with name '{}' found.", name)));
        }

        Ok(())
    }

    /// Sets the color of all lights to the given RGB color.
    pub fn set_all_by_color(&self, color: &str) -> Result<(), Box<Error>> {
        for (index, light) in &self.lights {
            if light.state.reachable {
                self.set_color_by_index_and_color(index, color)?;
            }
        }
        Ok(())
    }
}

/// Uses the meethue.com/api/nupnp to retreive the IP of the hue bridge.
pub fn get_hue_ip() -> Result<String, Box<Error>> {
    let body = reqwest::get("https://www.meethue.com/api/nupnp")?.text()?;
    let json: Value = serde_json::from_str(&body)?;

    Ok(json[0]["internalipaddress"].to_string().replace("\"", ""))
}

/// Loads the API token from $HOME/.config/rusty_hue/token.
fn get_token() -> Result<(String), Box<Error>> {
    match env::home_dir() {
        Some(path) => {
            let token_file = String::from(path.to_string_lossy()) + "/.config/rusty_hue/token";
            let mut f = File::open(token_file)?;

            let mut token = String::new();
            f.read_to_string(&mut token)?;
            token.truncate(40);
            return Ok(token);
        }
        None => Err(From::from("Failed to get home directory."))
    }

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
