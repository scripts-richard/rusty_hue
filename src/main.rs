#[macro_use]
extern crate clap;

extern crate rusty_hue;
use rusty_hue::hue::Hue;


fn main() {
    let matches = clap_app!(RustyHue =>
        (version: "0.2")
        (author: "Richard Mills <scripts.richard@gmail.com>")
        (about: "Control your Hue lights from the command line.")
        (@arg index: -i --index +takes_value "Select light by its index.")
        (@arg name: -n --name +takes_value "Select light by its name.")
        (@arg color: -c --color +takes_value "Set color by name (i.e. 'red').")
        (@subcommand info =>
            (about: "Displays information about Hue lights.")
            (version: "0.1")
        )
    ).get_matches();

    let hue = Hue::new().unwrap();

    let index = matches.value_of("index");
    let name = matches.value_of("name");
    let color = matches.value_of("color");

    match matches.subcommand_name() {
        Some("info") => {
            hue.print_info();
            return;
        },
        _ => ()
    }

    match (index, name, color) {
        (None, None, None) => {
            println!("Toggling lights...");
            if hue.toggle_lights().unwrap() {
                println!("Lights have been powered on.");
            } else {
                println!("Lights have been powered off.");
            }
        },

        (None, None, Some(c)) => {
            println!("Setting all lights to {}...", c);
            hue.set_all_by_color(c).unwrap();
        },

        (None, Some(n), None) => {
            println!("Toggling light '{}'...", n);
            if hue.toggle_by_name(n).unwrap() {
                println!("Light '{}' has been powered on.", n);
            } else {
                println!("Light '{}' has been powered off.", n);
            }
        },

        (None, Some(n), Some(c)) => {
            println!("Setting light '{}' to {}...", n, c);
            hue.set_color_by_name_and_color(n, c).unwrap();
        },

        (Some(i), None, None) => {
            println!("Toggling light at index: {}...", i);
            if hue.toggle_by_index(i).unwrap() {
                println!("Light at index: {} powered on.", i);
            } else {
                println!("Light at index: {} powered off.", i);
            }
        },

        (Some(i), None, Some(c)) => {
            println!("Setting light at index: {} to {}", i, c);
            hue.set_color_by_index_and_color(i, c).unwrap();
        },

        (Some(_), Some(_), None) => (),

        (Some(i), Some(n), Some(c)) => println!("Index: {}, name: {}, color: {}", i, n, c)
    }
}
