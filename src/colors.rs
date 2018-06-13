use serde_json;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub struct RGB {
    pub r: u8,
    pub g: u8,
    pub b: u8,
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

    pub fn xy_string(&self) -> String {
        format!("[{}, {}]", self.x, self.y)
    }

    pub fn adjust_for_gamut(&mut self, gamut: ColorGamut) {
        let gamut_point = GamutPoint { x: self.x, y: self.y };

        if gamut.point_in_gamut(&gamut_point) {
            return ();
        }

        let new_gamut_point = gamut.closest_point(&gamut_point);

        self.x = new_gamut_point.x;
        self.y = new_gamut_point.y;
    }
}

pub struct GamutPoint {
    x: f32,
    y: f32,
}

impl GamutPoint {
    fn sign(&self, p1: &GamutPoint, p2: &GamutPoint) -> bool {
        (self.x - p2.x) * (p1.y - p2.y) - (p1.x - p2.x) * (self.y - p2.y) < 0.0
    }

    fn closest_point_on_line(&self, p1: &GamutPoint, p2: &GamutPoint) -> GamutPoint {
        let mut k = (p2.y - p1.y) * (self.x - p1.x) - (p2.x - p1.x) * (self.y - p1.y);
        k /= (p2.y - p1.y).powi(2) + (p2.x - p1.x).powi(2);

        let x = self.x - k * (p2.y - p1.y);
        let y = self.y + k * (p2.x - p1.x);

        GamutPoint { x, y }
    }

    fn distance_to(&self, p: &GamutPoint) -> f32 {
        ((self.x - p.x).powi(2) + (self.y - p.y).powi(2)).sqrt()
    }
}

pub struct ColorGamut {
    red: GamutPoint,
    green: GamutPoint,
    blue: GamutPoint,
}

impl ColorGamut {
    pub fn point_in_gamut(&self, p: &GamutPoint) -> bool {
        let b1 = p.sign(&self.red, &self.green);
        let b2 = p.sign(&self.green, &self.blue);
        let b3 = p.sign(&self.blue, &self.red);

        (b1 == b2) && (b2 == b3)
    }

    pub fn closest_point(&self, p: &GamutPoint) -> GamutPoint {
        let proj1 = p.closest_point_on_line(&self.red, &self.green);
        let proj2 = p.closest_point_on_line(&self.green, &self.blue);
        let proj3 = p.closest_point_on_line(&self.blue, &self.red);

        let dist1 = p.distance_to(&proj1);
        let dist2 = p.distance_to(&proj2);
        let dist3 = p.distance_to(&proj3);

        if dist1 <= dist2 && dist1 <= dist3 {
            proj1
        } else if dist2 <= dist3 {
            proj2
        } else {
            proj3
        }
    }
}

pub const COLOR_GAMUT_A: ColorGamut = ColorGamut {
    red: GamutPoint { x: 0.704, y: 0.296 },
    green: GamutPoint { x: 0.2151, y: 0.7106 },
    blue: GamutPoint { x: 0.138, y: 0.08 }
};

pub const COLOR_GAMUT_B: ColorGamut = ColorGamut {
    red: GamutPoint { x: 0.675, y: 0.322 },
    green: GamutPoint { x: 0.409, y: 0.518 },
    blue: GamutPoint { x: 0.167, y: 0.04 }
};

pub const COLOR_GAMUT_C: ColorGamut = ColorGamut {
    red: GamutPoint { x: 0.692, y: 0.308 },
    green: GamutPoint { x: 0.17, y: 0.07 },
    blue: GamutPoint { x: 0.153, y: 0.048 }
};

pub fn color_gamut_lookup(model_id: &str) -> Option<char> {
    match model_id {
        "LST001" |
        "LLC005" |
        "LLC006" |
        "LLC007" |
        "LLC010" |
        "LLC011" |
        "LLC012" |
        "LLC013" |
        "LLC014" => Some('A'),
        "LCT001" |
        "LCT002" |
        "LCT003" |
        "LMM001" => Some('B'),
        "LCT010" |
        "LCT011" |
        "LCT014" |
        "LCT015" |
        "LCT016" |
        "LLC020" |
        "LST002" |
        "LCT012" => Some('C'),
        _ => None
    }
}

pub fn load_colors_from_file() -> Result<HashMap<String, RGB>, Box<Error>> {
    match env::home_dir() {
        Some(path) => {
            let colors_file = String::from(path.to_string_lossy()) + "/.config/rusty_hue/colors.json";
            let mut f = File::open(colors_file)?;

            let mut contents = String::new();
            f.read_to_string(&mut contents)?;

            let colors: HashMap<String, RGB> = serde_json::from_str(&contents)?;

            return Ok(colors);
        }
        None => Err(From::from("Failed to get home directory."))
    }
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
    fn point_in_triangle() {
        let point = GamutPoint{ x: 3.5, y: 1.5 };
        let triangle = ColorGamut{
            red: GamutPoint { x: 4.0, y: 1.0 },
            green: GamutPoint { x: 5.0, y: 3.0 },
            blue: GamutPoint { x: 2.0, y: 1.0 }
        };
        assert!(triangle.point_in_gamut(&point));
    }

    #[test]
    fn point_not_in_triangle() {
        let point = GamutPoint { x: 1.0, y: 3.0 };
        let triangle = ColorGamut{
            red: GamutPoint { x: 4.0, y: 1.0 },
            green: GamutPoint { x: 5.0, y: 3.0 },
            blue: GamutPoint { x: 2.0, y: 1.0 }
        };
        assert!(!triangle.point_in_gamut(&point));
    }

    #[test]
    fn closest_point_on_line() {
        let p1 = GamutPoint { x: 1.0, y: 2.0 };
        let p2 = GamutPoint { x: 2.0, y: 1.0 };
        let p3 = GamutPoint { x: 2.0, y: 3.0 };

        let new_point = p1.closest_point_on_line(&p2, &p3);

        assert_eq!(new_point.x, 2.0);
        assert_eq!(new_point.y, 2.0);
    }

    #[test]
    fn distance_between_points() {
        let p1 = GamutPoint { x: 1.0, y: 2.0 };
        let p2 = GamutPoint { x: 2.0, y: 2.0 };

        assert_eq!(p1.distance_to(&p2), 1.0);
    }

    #[test]
    fn closest_point_on_triangle() {
        let p = GamutPoint { x: 1.0, y: 2.0 };
        let triangle = ColorGamut {
            red: GamutPoint { x: 2.0, y: 1.0 },
            green: GamutPoint { x: 2.0, y: 3.0 },
            blue: GamutPoint { x: 3.0, y: 2.0 }
        };

        let new_point = triangle.closest_point(&p);

        assert_eq!(new_point.x, 2.0);//use serde_json::Value;
        assert_eq!(new_point.y, 2.0);
    }

    #[test]
    fn gamut_lookup() {
        assert_eq!(color_gamut_lookup("LLC007"), Some('A'));
        assert_eq!(color_gamut_lookup("LCT003"), Some('B'));
        assert_eq!(color_gamut_lookup("LST002"), Some('C'));
        assert_eq!(color_gamut_lookup("WRONG"), None);
    }

    #[test]
    fn load_colors_file() {
        let colors = load_colors_from_file();
        assert!(colors.is_ok());
        let colors = colors.unwrap();
        assert_eq!(colors["white"].r, 255);
    }
}
