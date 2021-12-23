use image::imageops::FilterType;
use image::io::Reader;
use image::DynamicImage;
use rand::distributions::WeightedIndex;
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use std::env;
use std::fs::File;
use std::io::{Cursor, Write};

#[derive(Copy, Clone)]
struct Color(u32, u32, u32);

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

fn write_color<T: Write>(color: Color) -> impl Fn(&mut T) {
    move |writer: &mut T| {
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
    let mut image = Vec::<u8>::new();

    generate_header(&mut image, image_width, image_height);

    for line in 0..image_height {
        let mut image_line = Vec::<Color>::with_capacity(image_width);
        image_line.resize(image_width, Color::default());

        if line % (sprite_height + margin) != 0 {
            for sprite_column in 0..sprite_columns {
                let start = sprite_column * (sprite_width + margin) + 1;
                let end = start + sprite_width;

                for column in start..(start + end + 1) / 2 {
                    // let color = colors[dist.sample(&mut rng)];
                    let color = colors[rng.sample(dist.clone())];

                    let index = column;
                    let sym_index = start + (end - 1 - column);

                    image_line[index] = color;
                    image_line[sym_index] = color;
                }
            }
        }

        for c in image_line.iter() {
            write_color(*c)(&mut image);
        }
    }

    let scale = 10;

    let image = read_image(image);
    let image = image::imageops::resize(
        &image,
        image_width as u32 * scale,
        image_height as u32 * scale,
        FilterType::Nearest,
    );

    image.save("image.png");
}
