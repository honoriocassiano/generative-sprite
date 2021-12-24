use std::env;
use std::io::{Cursor, Write};

use image::imageops::FilterType;
use image::io::Reader;
use image::{DynamicImage, ImageBuffer, Rgba};
use rand::distributions::WeightedIndex;
use rand::{thread_rng, Rng};

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

const PALETTES: [[Color; 6]; 2] = [
    [
        Color(0, 0, 0),
        Color(161, 69, 111),
        Color(59, 61, 221),
        Color(154, 207, 31),
        Color(28, 18, 228),
        Color(255, 214, 48),
    ],
    [
        Color(0, 0, 0),
        Color(246, 147, 26),
        Color(248, 100, 75),
        Color(250, 51, 126),
        Color(252, 28, 68),
        Color(255, 0, 0),
    ],
];

fn read_image<T: AsRef<[u8]>>(data: T) -> DynamicImage {
    Reader::new(Cursor::new(data))
        .with_guessed_format()
        .expect("Invalid format")
        .decode()
        .expect("Invalid format")
}

fn parse_palette_file(str: String) -> Vec<Vec<Color>> {
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
            let split = line.split(" ").collect::<Vec<_>>();

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

fn convert_index(width: usize) -> impl Fn(usize, usize) -> usize {
    move |line, column| width * line + column
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

fn main() {
    let args = parse_arguments(env::args().collect());

    let margin = 1;

    let sprite_width = args.sprite_width;
    let sprite_height = args.sprite_height;

    let sprite_columns = args.sprite_columns;
    let sprite_lines = args.sprite_lines;

    let image_width = sprite_width * sprite_columns + (sprite_columns + 1) * margin;
    let image_height = sprite_height * sprite_lines + (sprite_lines + 1) * margin;

    let mut image: Vec<Color> = (0..image_width * image_height)
        .into_iter()
        .map(|_| Color::default())
        .collect();

    let mut rng = thread_rng();
    let palettes: Vec<usize> = (0..sprite_lines * sprite_columns)
        .into_iter()
        .map(|_| rng.gen_range(0..PALETTES.len()))
        .collect();

    let index_converter = convert_index(image_width);
    let palette_index_converter = convert_index(sprite_columns);

    let dist = WeightedIndex::new(&[5, 1, 1, 1, 1, 1]).unwrap();

    for sprite_line in 0..sprite_lines {
        let start_line = sprite_line * (sprite_height + margin) + 1;
        let end_line = start_line + sprite_height;

        for line in start_line..end_line {
            for sprite_column in 0..sprite_columns {
                let start_column = sprite_column * (sprite_width + margin) + 1;
                let end_column = start_column + sprite_width;

                for column in start_column..(start_column + end_column + 1) / 2 {
                    let colors =
                        PALETTES[palettes[palette_index_converter(sprite_line, sprite_column)]];

                    let color = colors[rng.sample(dist.clone())];

                    let index = column;
                    let sym_index = start_column + (end_column - 1 - column);

                    image[index_converter(line, index)] = color;
                    image[index_converter(line, sym_index)] = color;
                }
            }
        }
    }

    let image = generate_image(image_width, image_height, image);

    image.save("image.png").expect("Unable to save image.png");
}

mod test {
    use crate::{parse_palette_file, Color};

    #[test]
    fn test_parse() {
        let str = "   \n  \n  1 2 3".to_owned();

        let expected = vec![vec![Color(1, 2, 3)]];

        let actual = parse_palette_file(str);

        assert_eq!(expected, actual);
    }
}
