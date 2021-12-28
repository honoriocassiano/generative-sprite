extern crate rand;

use std::env;
use std::fs::File;
use std::io::{Cursor, Read, Write};

use image::imageops::FilterType;
use image::io::Reader;
use image::{DynamicImage, ImageBuffer, Rgba};
use rand::distributions::{Distribution, WeightedIndex};
use rand::prelude::ThreadRng;
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

#[derive(Debug, Eq, PartialEq, Clone)]
struct Sprite {
    width: usize,
    height: usize,
    data: Vec<Color>,
}

impl Sprite {
    pub fn new(width: usize, height: usize, data: Vec<Color>) -> Self {
        Self {
            width,
            height,
            data,
        }
    }

    pub fn from_color(width: usize, height: usize, default_color: Color) -> Self {
        Self {
            width,
            height,
            data: vec![default_color].repeat(width * height),
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn data(&self) -> &Vec<Color> {
        &self.data
    }

    pub fn get_at(&self, line: usize, column: usize) -> Color {
        self.data[matrix_index_to_vec(self.width)(line, column)]
    }

    pub fn set_at(&mut self, line: usize, column: usize, color: Color) {
        self.data[matrix_index_to_vec(self.width)(line, column)] = color;
    }
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
    result
        .read_to_string(&mut content)
        .expect(format!("Unable to read {}", path).as_str());

    parse_palette_file(content)
}

fn generate_sprite(
    width: usize,
    height: usize,
    background: Color,
    palette: &[Color],
    mut rng: &mut ThreadRng,
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

fn generate_sprite_matrix(
    args: &Arguments,
    background: Color,
    palettes: &Vec<Vec<Color>>,
    mut rng: &mut ThreadRng,
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

fn generate_pixels(
    args: &Arguments,
    sprites: &Vec<Sprite>,
    margin: usize,
    background: Color,
) -> Vec<Color> {
    let sprite_height = args.sprite_height;
    let sprite_width = args.sprite_width;
    let sprite_columns = args.sprite_columns;
    let sprite_lines = args.sprite_lines;

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
    let width = sprite.width;
    let height = sprite.height;

    let mut vec = sprite.data().clone();

    let index_converter = matrix_index_to_vec(width);

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
                vec[index_converter(line, column)] = background;
            }
        }
    }

    Sprite::new(width, height, vec)
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

    let mut rng = thread_rng();

    let mut seed: [u8; 32] = Default::default();
    rng.fill(&mut seed);

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

    let image = generate_pixels(&args, &sprites, margin, background);
    let image = generate_image(image_width, image_height, image);

    let filename = format!("image_{}.png", seed);

    image.save(filename.clone()).expect("Unable to save file");

    println!("Saved file {}", filename);
}

#[allow(unused_imports)]
mod test {
    use std::fs::{remove_file, File};
    use std::io::Write;

    use uuid::Uuid;

    use crate::{
        matrix_index_to_vec, parse_palette_file, read_palettes, remove_lonely_pixels, Color, Sprite,
    };

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

    #[test]
    fn test_remove_lonely_pixels() {
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
}
