extern crate rusty_hue;

use rusty_hue::hue::Hue;

fn main() {
    let mut hue = Hue::new().unwrap();
    hue.toggle_lights().unwrap();
}
