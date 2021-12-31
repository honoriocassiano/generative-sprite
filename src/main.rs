extern crate rand;

use std::fs::File;
use std::io::{Cursor, Read, Write};

use image::imageops::FilterType;
use image::io::Reader;
use image::{DynamicImage, ImageBuffer, Rgba};
use rand::distributions::{Distribution, WeightedIndex};
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng, SeedableRng};
use regex::Regex;

use clap::{App, Arg};
use rand::rngs::StdRng;
use sprite::{Color, Sprite};
use std::convert::TryInto;

mod sprite;

#[derive(Copy, Clone)]
struct Size(u32, u32);

fn generate_header<T: Write>(writer: &mut T, width: usize, height: usize) {
    writer
        .write(format!("P3\n{} {}\n255", width, height).as_bytes())
        .expect("Unable to generate header");
}

fn write_color<T: Write>(writer: &mut T, color: Color) {
    writer
        .write(format!("\n{} {} {}", color.0, color.1, color.2).as_bytes())
        .expect("Unable to generate header");
}

fn read_image<T: AsRef<[u8]>>(data: T) -> DynamicImage {
    Reader::new(Cursor::new(data))
        .with_guessed_format()
        .expect("Invalid format")
        .decode()
        .expect("Invalid format")
}

fn parse_palette_file(str: String) -> Vec<Vec<Color>> {
    let only_spaces_regex = Regex::new(r"\s+").unwrap();
    let lines = str.lines().map(|l| l.trim()).collect::<Vec<_>>();

    let mut palettes = Vec::<Vec<Color>>::new();
    let mut palette = Vec::<Color>::new();

    for line in lines {
        if line.is_empty() {
            if !palette.is_empty() {
                palettes.push(palette.clone());
                palette.clear();
            }
        } else {
            let split = only_spaces_regex.split(line).collect::<Vec<_>>();

            let r = split[0].parse::<u8>().unwrap();
            let g = split[1].parse::<u8>().unwrap();
            let b = split[2].parse::<u8>().unwrap();

            palette.push(Color(r, g, b));
        }
    }

    if !palette.is_empty() {
        palettes.push(palette);
    }

    palettes
}

fn matrix_index_to_vec(width: usize) -> impl Fn(usize, usize) -> usize {
    assert!(width > 0);
    move |line, column| width * line + column
}

fn vec_index_to_matrix(width: usize) -> impl Fn(usize) -> (usize, usize) {
    assert!(width > 0);
    move |index| (index / width, index % width)
}

fn generate_image(
    image_width: usize,
    image_height: usize,
    image: Vec<Color>,
) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let mut image_bytes = Vec::<u8>::new();

    generate_header(&mut image_bytes, image_width, image_height);

    for c in image.iter() {
        write_color(&mut image_bytes, *c);
    }

    let image = read_image(image_bytes);

    let scale = 10;

    image::imageops::resize(
        &image,
        image_width as u32 * scale,
        image_height as u32 * scale,
        FilterType::Nearest,
    )
}

struct Arguments {
    pub sprite_width: usize,
    pub sprite_height: usize,
    pub sprite_columns: usize,
    pub sprite_lines: usize,

    pub margin: usize,

    pub seed: Option<[u8; 32]>,
}

fn parse_arguments() -> Arguments {
    let matches = App::new("Generative")
        .version("0.1.0")
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
        .get_matches();

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

    let seed = matches.value_of("seed").map(|h| parse_seed(h));

    Arguments {
        sprite_width,
        sprite_height,
        sprite_lines,
        sprite_columns,
        margin,
        seed,
    }
}

fn parse_seed(hash: &str) -> [u8; 32] {
    assert_eq!(hash.len(), 64, "Seed must have 32 bits");

    let vec = hash
        .chars()
        .collect::<Vec<_>>()
        .chunks(2)
        .map(|c| {
            let hex_number = c.iter().collect::<String>();

            u8::from_str_radix(hex_number.as_str(), 16)
                .expect(format!("Invalid byte '{}'", hex_number).as_str())
        })
        .collect::<Vec<_>>();

    vec.try_into().unwrap()
}

fn read_palettes(path: &str) -> Vec<Vec<Color>> {
    let mut result = File::open(path).expect("Cannot read file");

    let mut content = String::new();
    result
        .read_to_string(&mut content)
        .expect(format!("Unable to read {}", path).as_str());

    parse_palette_file(content)
}

fn generate_sprite<R: Rng>(
    width: usize,
    height: usize,
    background: Color,
    palette: &[Color],
    mut rng: &mut R,
) -> Sprite {
    let data = (0..height)
        .into_iter()
        .flat_map(|_| {
            let mut image_line = vec![background].repeat(width);

            for column in 0..(width + 1) / 2 {
                let index = column;
                let sym_index = width - 1 - column;

                let factor = (sym_index - index) as f32 / width as f32;

                let weights =
                    WeightedIndex::new(&[0.5 - 0.5 * factor, 0.5 + 0.5 * factor]).unwrap();

                let values = [true, false];

                if values[weights.sample(&mut rng)] {
                    // TODO Re-add weights
                    let color = *palette.choose(&mut rng).unwrap();

                    image_line[index] = color;
                    image_line[sym_index] = color;
                }
            }

            image_line
        })
        .collect::<Vec<_>>();

    Sprite::new(width, height, data)
}

fn generate_sprite_matrix<R: Rng>(
    args: &Arguments,
    background: Color,
    palettes: &Vec<Vec<Color>>,
    mut rng: &mut R,
) -> Vec<Sprite> {
    let sprite_height = args.sprite_height;
    let sprite_width = args.sprite_width;
    let sprite_columns = args.sprite_columns;
    let sprite_lines = args.sprite_lines;

    (0..sprite_columns * sprite_lines)
        .into_iter()
        .map(|_| {
            let palette = palettes.choose(&mut rng).unwrap();
            generate_sprite(sprite_width, sprite_height, background, palette, &mut rng)
        })
        .collect()
}

fn generate_pixels(args: &Arguments, sprites: &Vec<Sprite>, background: Color) -> Vec<Color> {
    let sprite_height = args.sprite_height;
    let sprite_width = args.sprite_width;
    let sprite_columns = args.sprite_columns;
    let sprite_lines = args.sprite_lines;
    let margin = args.margin;

    let image_width = sprite_width * sprite_columns + (sprite_columns + 1) * margin;
    let image_height = sprite_height * sprite_lines + (sprite_lines + 1) * margin;

    let mut image = vec![background].repeat(image_width * image_height);

    let image_index_converter = matrix_index_to_vec(image_width);
    let sprite_index_converter = vec_index_to_matrix(sprite_columns);

    for sprite_index in 0..sprites.len() {
        let (sprite_line, sprite_column) = sprite_index_converter(sprite_index);

        let start_line = sprite_line * (sprite_height + margin) + margin;
        let start_column = sprite_column * (sprite_width + margin) + margin;

        let sprite = &sprites[sprite_index];

        for sc in 0..sprite_width {
            for sl in 0..sprite_height {
                let l = start_line + sl;
                let c = start_column + sc;

                if (l < image_height) && (c < image_width) {
                    image[image_index_converter(l, c)] = sprite.get_at(sl, sc);
                }
            }
        }
    }

    image
}

fn remove_lonely_pixels(
    sprite: &Sprite,
    margin: usize,
    min_count: u32,
    background: Color,
) -> Sprite {
    let width = sprite.width();
    let height = sprite.height();

    let mut new_sprite = sprite.clone();

    for line in 0..height {
        for column in 0..width {
            let start_line = line - (line.min(margin));
            let end_line = (line + margin + 1).min(height);

            let start_column = column - (column.min(margin));
            let end_column = (column + margin + 1).min(width);

            let count = (start_line..end_line).into_iter().fold(0u32, |acc, l| {
                (start_column..end_column)
                    .into_iter()
                    .fold(0u32, |acc2, c| {
                        if sprite.get_at(l, c) != background {
                            acc2 + 1
                        } else {
                            acc2
                        }
                    })
                    + acc
            });

            if count < min_count {
                new_sprite.set_at(line, column, background);
            }
        }
    }

    new_sprite
}

fn main() {
    let args = parse_arguments();

    let background = Color::default();

    let sprite_width = args.sprite_width;
    let sprite_height = args.sprite_height;

    let sprite_columns = args.sprite_columns;
    let sprite_lines = args.sprite_lines;
    let margin = args.margin;

    let image_width = sprite_width * sprite_columns + (sprite_columns + 1) * margin;
    let image_height = sprite_height * sprite_lines + (sprite_lines + 1) * margin;

    let palettes = read_palettes("palettes");

    let seed = match args.seed {
        Some(s) => s,
        None => {
            let mut rng = thread_rng();
            let mut temp: [u8; 32] = Default::default();

            rng.fill(&mut temp);

            temp
        }
    };

    let mut rng = StdRng::from_seed(seed);

    let seed = seed
        .iter()
        .map(|t| format!("{:02x}", t))
        .collect::<Vec<_>>()
        .join("");

    let sprites = generate_sprite_matrix(&args, background, &palettes, &mut rng)
        .into_iter()
        .map(|s| remove_lonely_pixels(&s, 2, 8, background))
        .map(|s| remove_lonely_pixels(&s, 2, 4, background))
        .collect::<Vec<_>>();

    let image = generate_pixels(&args, &sprites, background);
    let image = generate_image(image_width, image_height, image);

    let filename = format!("image_{}.png", seed);

    image.save(filename.clone()).expect("Unable to save file");

    println!("Saved file {}", filename);
}

#[cfg(test)]
mod test {

    #[test]
    #[should_panic]
    fn test_matrix_index_to_vec_width_zero() {
        use crate::matrix_index_to_vec;

        matrix_index_to_vec(0)(1, 2);
    }

    #[test]
    fn test_matrix_index_to_vec() {
        use crate::matrix_index_to_vec;

        let converter = matrix_index_to_vec(2);

        assert_eq!(0, converter(0, 0));
        assert_eq!(2, converter(1, 0));
        assert_eq!(1, converter(0, 1));
    }

    #[test]
    fn test_parse() {
        use crate::parse_palette_file;

        use crate::sprite::Color;

        let str = "   \n  \n  1 \t2    3".to_owned();

        let expected = vec![vec![Color(1, 2, 3)]];

        let actual = parse_palette_file(str);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_parse_file() {
        use std::fs::{remove_file, File};
        use std::io::Write;

        use uuid::Uuid;

        use crate::read_palettes;
        use crate::sprite::{Color, Sprite};

        let str = "   \n  \n  1 \t2    3";

        let path = Uuid::new_v4().to_string();
        let mut file = File::create(path.as_str()).unwrap();
        file.write(str.as_bytes()).unwrap();

        let expected = vec![vec![Color(1, 2, 3)]];

        let actual = read_palettes(path.as_str());

        assert_eq!(expected, actual);

        remove_file(path).unwrap();
    }

    #[test]
    fn test_remove_lonely_pixels() {
        use crate::{remove_lonely_pixels, Color, Sprite};

        let width = 5;
        let height = 5;

        let vec_size = width * height;

        let expected = Sprite::new(width, height, vec![Color::default()].repeat(width * height));

        let data = {
            let mut vec2 = vec![Color::default()].repeat(vec_size);

            vec2[13] = Color(255, 0, 0);

            vec2
        };

        let image = Sprite::new(width, height, data);

        let actual = remove_lonely_pixels(&image, 2, 8, Color::default());

        assert_eq!(actual, expected);
    }

    #[test]
    fn should_generate_solid_color_sprite() {
        use crate::{Color, Sprite};

        let width = 5;
        let height = 5;
        let color = Color(255, 0, 0);
        let expected = vec![color].repeat(width * height);

        let sprite = Sprite::from_color(width, height, color);

        assert_eq!(*sprite.data(), expected);
    }

    #[test]
    fn should_parse_seed() {
        use crate::parse_seed;

        let seed = "04ed394c85de2fe0f1b778d37cc029b6a1366f1aa26498fb123b4ac75d955e08";
        let expected: [u8; 32] = [
            0x04, 0xed, 0x39, 0x4c, 0x85, 0xde, 0x2f, 0xe0, 0xf1, 0xb7, 0x78, 0xd3, 0x7c, 0xc0,
            0x29, 0xb6, 0xa1, 0x36, 0x6f, 0x1a, 0xa2, 0x64, 0x98, 0xfb, 0x12, 0x3b, 0x4a, 0xc7,
            0x5d, 0x95, 0x5e, 0x08,
        ];

        let actual = parse_seed(seed);

        assert_eq!(actual, expected);
    }
}
