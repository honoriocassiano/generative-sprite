use clap::{App, Arg};
use std::ffi::OsString;

pub struct Arguments {
    pub sprite_width: usize,
    pub sprite_height: usize,
    pub sprite_columns: usize,
    pub sprite_lines: usize,

    pub margin: usize,

    pub seed: Option<[u8; 32]>,
}

pub fn parse_arguments<I, T>(args: I) -> Arguments
where
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

#[cfg(test)]
mod test {
    use crate::argparser::parse_arguments;

    #[test]
    fn should_parse_arguments_with_default_margin() {
        let arg_list = vec!["generative", "1", "2", "3", "4"];

        let args = parse_arguments(arg_list);

        assert_eq!(1, args.sprite_width);
        assert_eq!(2, args.sprite_height);
        assert_eq!(3, args.sprite_columns);
        assert_eq!(4, args.sprite_lines);
        assert_eq!(2, args.margin);
        assert_eq!(None, args.seed);
    }

    #[test]
    fn should_parse_arguments_with_margin() {
        let arg_list = vec!["generative", "1", "2", "3", "4", "-m", "5"];

        let args = parse_arguments(arg_list);

        assert_eq!(1, args.sprite_width);
        assert_eq!(2, args.sprite_height);
        assert_eq!(3, args.sprite_columns);
        assert_eq!(4, args.sprite_lines);
        assert_eq!(5, args.margin);
        assert_eq!(None, args.seed);
    }

    #[test]
    fn should_parse_arguments_with_seed() {
        let arg_list = vec![
            "generative",
            "1",
            "2",
            "3",
            "4",
            "-s",
            "f7b028003248f3ca4df45566c21edffc03629f488da3639b90e1f9566bcd8b62",
        ];

        let seed: [u8; 32] = [
            0xf7, 0xb0, 0x28, 0x00, 0x32, 0x48, 0xf3, 0xca, 0x4d, 0xf4, 0x55, 0x66, 0xc2, 0x1e,
            0xdf, 0xfc, 0x03, 0x62, 0x9f, 0x48, 0x8d, 0xa3, 0x63, 0x9b, 0x90, 0xe1, 0xf9, 0x56,
            0x6b, 0xcd, 0x8b, 0x62,
        ];
        let args = parse_arguments(arg_list);

        assert_eq!(1, args.sprite_width);
        assert_eq!(2, args.sprite_height);
        assert_eq!(3, args.sprite_columns);
        assert_eq!(4, args.sprite_lines);
        assert_eq!(2, args.margin);
        assert_eq!(Some(seed), args.seed);
    }
}
