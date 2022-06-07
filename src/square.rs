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

#[deprecated]
pub trait _BoardPos: Debug {
    fn idx(&self) -> usize;
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

impl From<Square> for usize {
    fn from(square: Square) -> Self {
        square as usize
    }
}

impl TryFrom<usize> for Square {
    type Error = String;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
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
            .get(value)
            .ok_or_else(|| {
                format!(
                    "index '{}' is not valid, make sure it's in the range '0..64'",
                    value
                )
            })
            .map(|s| *s)
    }
}

impl _BoardPos for usize {
    fn idx(&self) -> usize {
        *self
    }
}

impl _BoardPos for Square {
    fn idx(&self) -> usize {
        (*self).into()
    }
}
