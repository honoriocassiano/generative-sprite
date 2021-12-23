use std::env;
use std::io::{Cursor, Write};

use image::imageops::FilterType;
use image::io::Reader;
use image::{DynamicImage, ImageBuffer, Rgba};
use rand::distributions::WeightedIndex;
use rand::{thread_rng, Rng};

#[derive(Copy, Clone)]
struct Color(u8, u8, u8);

#[derive(Copy, Clone)]
struct Size(u32, u32);

impl Default for Color {
    fn default() -> Self {
        Self(0, 0, 0)
    }
}

fn generate_header<T: Write>(writer: &mut T, width: usize, height: usize) {
    writer.write(format!("P3\n{} {}\n255", width, height).as_bytes());
}

fn write_color<'a, T: 'a + Write>(writer: &'a mut T) -> impl FnMut(Color) + 'a {
    move |color: Color| {
        writer.write(format!("\n{} {} {}", color.0, color.1, color.2).as_bytes());
    }
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
        write_color(&mut image_bytes)(*c);
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

fn main() {
    let args: Vec<String> = env::args().collect();

    let sprite_width = args[1].parse::<usize>().unwrap();
    let sprite_height = args[2].parse::<usize>().unwrap();

    let margin: usize = 1;

    let sprite_columns = args[3].parse::<usize>().unwrap();
    let sprite_lines = args[4].parse::<usize>().unwrap();

    let image_width = sprite_width * sprite_columns + (sprite_columns + 1) * margin;
    let image_height = sprite_height * sprite_lines + (sprite_lines + 1) * margin;

    let colors = PALETTES[1];

    let color_weights = [5, 1, 1, 1, 1, 1];

    let dist = WeightedIndex::new(&color_weights).unwrap();

    let mut rng = thread_rng();

    let mut image = Vec::<Color>::with_capacity(image_width * image_height);
    image.resize(image_width * image_height, Color::default());

    let index_converter = convert_index(image_width);

    for line in 0..image_height {
        if line % (sprite_height + margin) != 0 {
            for sprite_column in 0..sprite_columns {
                let start = sprite_column * (sprite_width + margin) + 1;
                let end = start + sprite_width;

                for column in start..(start + end + 1) / 2 {
                    let color = colors[rng.sample(dist.clone())];

                    let index = column;
                    let sym_index = start + (end - 1 - column);

                    image[index_converter(line, index)] = color;
                    image[index_converter(line, sym_index)] = color;
                }
            }
        }
    }

    let image = generate_image(image_width, image_height, image);

    image.save("image.png");
}
