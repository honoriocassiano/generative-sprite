use std::env;
use std::fs::File;
use std::io::{Cursor, Read, Write};

use image::imageops::FilterType;
use image::io::Reader;
use image::{DynamicImage, ImageBuffer, Rgba};
use rand::distributions::WeightedIndex;
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use regex::Regex;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
struct Color(u8, u8, u8);

#[derive(Copy, Clone)]
struct Size(u32, u32);

impl Default for Color {
    fn default() -> Self {
        Self(0, 0, 0)
    }
}

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
}

fn parse_arguments(args: Vec<String>) -> Arguments {
    let sprite_width = args[1].parse::<usize>().unwrap();
    let sprite_height = args[2].parse::<usize>().unwrap();
    let sprite_columns = args[3].parse::<usize>().unwrap();
    let sprite_lines = args[4].parse::<usize>().unwrap();

    Arguments {
        sprite_width,
        sprite_height,
        sprite_lines,
        sprite_columns,
    }
}

fn read_palettes(path: &str) -> Vec<Vec<Color>> {
    let mut result = File::open(path).expect("Cannot read file");

    let mut content = String::new();
    result.read_to_string(&mut content);

    parse_palette_file(content)
}

type Sprite = Vec<Vec<Color>>;

fn generate_sprite(width: usize, height: usize, background: Color, palette: &[Color]) -> Sprite {
    let mut rng = thread_rng();

    (0..height)
        .into_iter()
        .map(|_| {
            let mut image_line = Vec::with_capacity(width);
            image_line.resize(width, background);

            for column in 0..(width + 1) / 2 {
                if *[true, false].choose(&mut rng).unwrap() {
                    // TODO Re-add weights
                    let color = *palette.choose(&mut rng).unwrap();

                    let index = column;
                    let sym_index = (width - 1 - column);

                    image_line[index] = color;
                    image_line[sym_index] = color;
                }
            }

            image_line
        })
        .collect::<Vec<_>>()
}

fn generate_sprite_matrix(
    args: &Arguments,
    background: Color,
    palettes: Vec<Vec<Color>>,
) -> Vec<Sprite> {
    let sprite_height = args.sprite_height;
    let sprite_width = args.sprite_width;
    let sprite_columns = args.sprite_columns;
    let sprite_lines = args.sprite_lines;

    let mut rng = thread_rng();

    (0..sprite_columns * sprite_lines)
        .into_iter()
        .map(|_| {
            generate_sprite(
                sprite_width,
                sprite_height,
                background,
                palettes.choose(&mut rng).unwrap(),
            )
        })
        .collect()
}

fn generate_pixels(
    args: &Arguments,
    margin: usize,
    background: Color,
    palettes: Vec<Vec<Color>>,
) -> Vec<Color> {
    let sprite_height = args.sprite_height;
    let sprite_width = args.sprite_width;
    let sprite_columns = args.sprite_columns;
    let sprite_lines = args.sprite_lines;

    let image_width = sprite_width * sprite_columns + (sprite_columns + 1) * margin;
    let image_height = sprite_height * sprite_lines + (sprite_lines + 1) * margin;

    let sprites = generate_sprite_matrix(&args, background, palettes);

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
                    image[image_index_converter(l, c)] = sprite[sl][sc];
                }
            }
        }
    }

    image
}

fn main() {
    let args = parse_arguments(env::args().collect());

    let margin = 2;
    let background = Color::default();

    let sprite_width = args.sprite_width;
    let sprite_height = args.sprite_height;

    let sprite_columns = args.sprite_columns;
    let sprite_lines = args.sprite_lines;

    let image_width = sprite_width * sprite_columns + (sprite_columns + 1) * margin;
    let image_height = sprite_height * sprite_lines + (sprite_lines + 1) * margin;

    let palettes = read_palettes("palettes");

    let image = generate_pixels(&args, margin, background, palettes);
    let image = generate_image(image_width, image_height, image);

    image.save("image.png").expect("Unable to save image.png");
}

mod test {
    use std::fs::{remove_file, File};
    use std::io::Write;

    use uuid::Uuid;

    use crate::{matrix_index_to_vec, parse_palette_file, read_palettes, Color};

    #[test]
    #[should_panic]
    fn test_matrix_index_to_vec_width_zero() {
        matrix_index_to_vec(0)(1, 2);
    }

    #[test]
    fn test_matrix_index_to_vec() {
        let converter = matrix_index_to_vec(2);

        assert_eq!(0, converter(0, 0));
        assert_eq!(2, converter(1, 0));
        assert_eq!(1, converter(0, 1));
    }

    #[test]
    fn test_parse() {
        let str = "   \n  \n  1 \t2    3".to_owned();

        let expected = vec![vec![Color(1, 2, 3)]];

        let actual = parse_palette_file(str);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_parse_file() {
        let str = "   \n  \n  1 \t2    3";

        let path = Uuid::new_v4().to_string();
        let mut file = File::create(path.as_str()).unwrap();
        file.write(str.as_bytes()).unwrap();

        let expected = vec![vec![Color(1, 2, 3)]];

        let actual = read_palettes(path.as_str());

        assert_eq!(expected, actual);

        remove_file(path);
    }
}
