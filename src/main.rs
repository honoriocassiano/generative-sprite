extern crate rand;

use std::fs::File;
use std::io::Read;

use image::imageops::FilterType;
use image::{ImageBuffer, Rgb};
use rand::distributions::{Distribution, WeightedIndex};
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use regex::Regex;

use crate::argparser::Arguments;
use crate::seed::Seed;
use rand::rngs::StdRng;
use sprite::{Color, Sprite};

mod argparser;
mod seed;
mod sprite;

#[derive(Copy, Clone)]
struct Size(u32, u32);

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
    pixels: Vec<Color>,
) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let index_converter = matrix_index_to_vec(image_width);

    let image = ImageBuffer::from_fn(image_width as u32, image_height as u32, |x, y| {
        image::Rgb(pixels[index_converter(x as usize, y as usize)].into())
    });

    let scale = 10;

    image::imageops::resize(
        &image,
        image_width as u32 * scale,
        image_height as u32 * scale,
        FilterType::Nearest,
    )
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
    let args = argparser::parse_arguments(std::env::args_os());

    let background = Color::default();

    let sprite_width = args.sprite_width;
    let sprite_height = args.sprite_height;

    let sprite_columns = args.sprite_columns;
    let sprite_lines = args.sprite_lines;
    let margin = args.margin;

    let image_width = sprite_width * sprite_columns + (sprite_columns + 1) * margin;
    let image_height = sprite_height * sprite_lines + (sprite_lines + 1) * margin;

    let palettes = read_palettes("palettes");

    let seed = match &args.seed {
        Some(s) => s.clone(),
        None => Seed::default(),
    };

    let mut rng = StdRng::from_seed(seed.data());

    let sprites = generate_sprite_matrix(&args, background, &palettes, &mut rng).into_iter();
    let sprites = if args.sprite_width > 9 && args.sprite_height > 9 {
        sprites
            .map(|s| remove_lonely_pixels(&s, 2, 8, background))
            .map(|s| remove_lonely_pixels(&s, 2, 4, background))
            .collect::<Vec<_>>()
    } else {
        sprites.collect::<Vec<_>>()
    };

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
        use crate::sprite::Color;

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
}
