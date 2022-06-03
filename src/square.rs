use std::fmt::Debug;

#[rustfmt::skip]
#[derive(Debug, Clone, Copy)]
pub enum Square {
    A8, B8, C8, D8, E8, F8, G8, H8,
    A7, B7, C7, D7, E7, F7, G7, H7,
    A6, B6, C6, D6, E6, F6, G6, H6,
    A5, B5, C5, D5, E5, F5, G5, H5,
    A4, B4, C4, D4, E4, F4, G4, H4,
    A3, B3, C3, D3, E3, F3, G3, H3,
    A2, B2, C2, D2, E2, F2, G2, H2,
    A1, B1, C1, D1, E1, F1, G1, H1,
}

pub trait BoardPos: Debug {
    fn idx(&self) -> i8;
}

impl From<Square> for i8 {
    fn from(square: Square) -> Self {
        square as i8
    }
}

impl From<Square> for u64 {
    fn from(square: Square) -> Self {
        square as u64
    }
}

impl TryFrom<u64> for Square {
    type Error = String;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        use Square::*;

        #[rustfmt::skip]
        const LOOKUP: [Square; 64] = [
            A8, B8, C8, D8, E8, F8, G8, H8,
            A7, B7, C7, D7, E7, F7, G7, H7,
            A6, B6, C6, D6, E6, F6, G6, H6,
            A5, B5, C5, D5, E5, F5, G5, H5,
            A4, B4, C4, D4, E4, F4, G4, H4,
            A3, B3, C3, D3, E3, F3, G3, H3,
            A2, B2, C2, D2, E2, F2, G2, H2,
            A1, B1, C1, D1, E1, F1, G1, H1,
        ];

        LOOKUP
            .get(value as usize)
            .ok_or_else(|| {
                format!(
                    "index '{}' is not valid, make sure it's in the range '0..64'",
                    value
                )
            })
            .map(|s| *s)
    }
}

impl BoardPos for i8 {
    fn idx(&self) -> i8 {
        *self
    }
}

impl BoardPos for u64 {
    fn idx(&self) -> i8 {
        (*self) as i8
    }
}

impl BoardPos for usize {
    fn idx(&self) -> i8 {
        (*self) as i8
    }
}

impl BoardPos for Square {
    fn idx(&self) -> i8 {
        (*self).into()
    }
}
