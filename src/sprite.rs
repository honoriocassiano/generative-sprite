#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Color(pub u8, pub u8, pub u8);

impl Default for Color {
    fn default() -> Self {
        Self(0, 0, 0)
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Sprite {
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
        self.data[crate::matrix_index_to_vec(self.width)(line, column)]
    }

    pub fn set_at(&mut self, line: usize, column: usize, color: Color) {
        self.data[crate::matrix_index_to_vec(self.width)(line, column)] = color;
    }
}

#[cfg(test)]
mod test {
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
}
