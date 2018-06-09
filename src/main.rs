#[macro_use]
extern crate clap;

extern crate rusty_hue;
use rusty_hue::hue::Hue;


fn main() {
    let matches = clap_app!(RustyHue =>
        (version: "0.1")
        (author: "Richard Mills <scripts.richard@gmail.com>")
        (about: "Control your Hue lights from the command line.")
        (@subcommand info =>
            (about: "Displays information about Hue lights.")
            (version: "0.1")
        )
    ).get_matches();

    let mut hue = Hue::new().unwrap();

    match matches.subcommand_name() {
        Some("info") => hue.print_info(),
        _ => hue.toggle_lights().unwrap()
    }
}
