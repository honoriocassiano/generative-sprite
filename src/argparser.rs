use std::ffi::OsString;
use clap::{App, Arg};

pub struct Arguments {
    pub sprite_width: usize,
    pub sprite_height: usize,
    pub sprite_columns: usize,
    pub sprite_lines: usize,

    pub margin: usize,

    pub seed: Option<[u8; 32]>,
}

pub fn parse_arguments<I, T>(args: I) -> Arguments where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let matches = App::new("Generative")
        .version(clap::crate_version!())
        .about("Generate random sprites")
        .arg(
            Arg::with_name("sprite-width")
                .help("Width (in pixels) by sprite")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("sprite-height")
                .help("Height (in pixels) by sprite")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("sprite-columns")
                .help("Number of columns of sprites matrix")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("sprite-lines")
                .help("Number of lines of sprites matrix")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("margin")
                .help("Size of margin between sprites")
                .default_value("2")
                .short("m")
                .long("margin")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("seed")
                .help("Seed to use")
                .short("s")
                .long("seed")
                .takes_value(true),
        )
        .get_matches_from(args);

    let sprite_width = matches
        .value_of("sprite-width")
        .unwrap()
        .parse::<usize>()
        .expect("Invalid sprite-width");
    let sprite_height = matches
        .value_of("sprite-height")
        .unwrap()
        .parse::<usize>()
        .expect("Invalid sprite-height");
    let sprite_columns = matches
        .value_of("sprite-columns")
        .unwrap()
        .parse::<usize>()
        .expect("Invalid sprite-columns");
    let sprite_lines = matches
        .value_of("sprite-lines")
        .unwrap()
        .parse::<usize>()
        .expect("Invalid sprite-lines");
    let margin = match matches.value_of("margin") {
        None => 2,
        Some(m) => m.parse::<usize>().expect("Invalid margin"),
    };

    let seed = matches.value_of("seed").map(|h| crate::parse_seed(h));

    Arguments {
        sprite_width,
        sprite_height,
        sprite_lines,
        sprite_columns,
        margin,
        seed,
    }
}
