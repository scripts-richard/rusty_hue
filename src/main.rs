extern crate rusty_hue;

use rusty_hue::hue::Hue;

fn main() {
    let mut hue = Hue::new().unwrap();
    hue.get_lights().unwrap();
}
