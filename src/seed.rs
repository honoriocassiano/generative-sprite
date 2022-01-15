use crate::seed::SeedParseError::{InvalidByte, InvalidSize};
use rand::{thread_rng, Rng};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Seed {
    data: [u8; 32],
}

impl Seed {
    pub fn new(data: [u8; 32]) -> Self {
        Self { data }
    }

    pub fn data(&self) -> [u8; 32] {
        self.data
    }
}

impl Default for Seed {
    fn default() -> Self {
        let mut rng = thread_rng();
        let mut data: [u8; 32] = Default::default();

        rng.fill(&mut data);

        Self { data }
    }
}

#[derive(Debug)]
pub enum SeedParseError {
    InvalidSize,
    InvalidByte(String),
}

impl FromStr for Seed {
    type Err = SeedParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let hash: String = value.into();

        if hash.len() != 64 {
            return Err(InvalidSize);
        }

        let vec = hash.chars().collect::<Vec<_>>();

        let mut data: [u8; 32] = Default::default();

        for (i, c) in vec.chunks(2).into_iter().enumerate() {
            let hex_number = c.iter().collect::<String>();

            data[i] =
                u8::from_str_radix(hex_number.as_str(), 16).map_err(|_| InvalidByte(hex_number))?
        }

        let seed = Self { data };

        Ok(seed)
    }
}

impl Display for Seed {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let string = self
            .data
            .iter()
            .map(|t| format!("{:02x}", t))
            .collect::<Vec<_>>()
            .join("");

        f.write_str(string.as_str())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn should_parse_seed() {
        let seed = "04ed394c85de2fe0f1b778d37cc029b6a1366f1aa26498fb123b4ac75d955e08";
        let expected = [
            0x04, 0xed, 0x39, 0x4c, 0x85, 0xde, 0x2f, 0xe0, 0xf1, 0xb7, 0x78, 0xd3, 0x7c, 0xc0,
            0x29, 0xb6, 0xa1, 0x36, 0x6f, 0x1a, 0xa2, 0x64, 0x98, 0xfb, 0x12, 0x3b, 0x4a, 0xc7,
            0x5d, 0x95, 0x5e, 0x08,
        ];
        let expected = Seed::new(expected);

        let actual = Seed::from_str(seed).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn should_generate_random_seed() {
        assert_ne!(Seed::default(), Seed::default());
    }
}
