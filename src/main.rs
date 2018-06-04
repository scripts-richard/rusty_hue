extern crate rusty_hue;

use rusty_hue::*;

fn main() {
    let mut hue = Hue::new().unwrap();
    hue.get_lights().unwrap();
}
