#[macro_use]
extern crate clap;

extern crate rusty_hue;
use rusty_hue::hue::Hue;


fn main() {
    let matches = clap_app!(RustyHue =>
        (version: "0.3")
        (author: "Richard Mills <scripts.richard@gmail.com>")
        (about: "Control your Hue lights from the command line.")
        (@arg index: -i --index +takes_value "Select light by its index.")
        (@arg name: -n --name +takes_value "Select light by its name.")
        (@subcommand color =>
            (about: "Set color by name (i.e. 'red').")
            (version: "0.1")
            (@arg COLOR: +required "Color to be set.")
        )
        (@subcommand info =>
            (about: "Displays information about Hue lights.")
            (version: "0.1")
        )
        (@subcommand rename =>
            (about: "Change a light's configuration.")
            (version: "0.1")
            (@arg INDEX: +required "Index of light to set value.")
            (@arg NAME: +required "New name value for light.")
        )
    ).get_matches();

    let hue = Hue::new().unwrap();

    let index = matches.value_of("index");
    let name = matches.value_of("name");

    match matches.subcommand_name() {
        Some("color") => {
            if let Some(matches) = matches.subcommand_matches("color") {
                if let Some(color) = matches.value_of("COLOR") {
                    match (index, name) {
                        (None, None) => {
                            println!("Setting all lights to {}...", color);
                            hue.set_all_by_color(color).unwrap();
                        },
                        (None, Some(name)) => {
                            println!("Setting light '{}' to {}...", name, color);
                            hue.set_color_by_name_and_color(name, color).unwrap();
                        },
                        (Some(index), None) => {
                            println!("Setting light at index: {} to {}", index, color);
                            hue.set_color_by_index_and_color(index, color).unwrap();
                        },
                        (Some(index), Some(name)) => {
                            println!("Setting light at index: {} to {}", index, color);
                            hue.set_color_by_index_and_color(index, color).unwrap();

                            println!("Setting light '{}' to {}...", name, color);
                            hue.set_color_by_name_and_color(name, color).unwrap();
                        },
                    }
                }
            }
            return;
        },

        Some("info") => {
            hue.print_info();
            return;
        },

        Some("rename") => {
            if let Some(matches) = matches.subcommand_matches("rename") {
                let index = matches.value_of("INDEX");
                let name = matches.value_of("NAME");

                match (index, name) {
                    (Some(index), Some(name)) => {
                        hue.rename_light(index, name).unwrap();
                    },
                    _ => () // Any other condition besides the above will be caught by the arg parser.
                }
            }
            return;
        },

        _ => {
            match (index, name) {
                (None, None) => {
                    println!("Toggling lights...");
                    if hue.toggle_lights().unwrap() {
                        println!("Lights have been powered on.");
                    } else {
                        println!("Lights have been powered off.");
                    }
                },
                (None, Some(name)) => {
                    println!("Toggling light '{}'...", name);
                    if hue.toggle_by_name(name).unwrap() {
                        println!("Light '{}' has been powered on.", name);
                    } else {
                        println!("Light '{}' has been powered off.", name);
                    }
                },
                (Some(index), None) => {
                    println!("Toggling light at index: {}...", index);
                    if hue.toggle_by_index(index).unwrap() {
                        println!("Light at index: {} powered on.", index);
                    } else {
                        println!("Light at index: {} powered off.", index);
                    }
                },
                (Some(index), Some(name)) => {
                    println!("Toggling light at index: {}...", index);
                    if hue.toggle_by_index(index).unwrap() {
                        println!("Light at index: {} powered on.", index);
                    } else {
                        println!("Light at index: {} powered off.", index);
                    }

                    println!("Toggling light '{}'...", name);
                    if hue.toggle_by_name(name).unwrap() {
                        println!("Light '{}' has been powered on.", name);
                    } else {
                        println!("Light '{}' has been powered off.", name);
                    }
                }
            }
        }
    }
}
