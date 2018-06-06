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
}
