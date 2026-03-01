use std::fmt;
use std::fmt::{Debug, Display};
use image::{Pixel, Rgba};

/// A unique identifier for a tile that lives in a Vec.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct TileId(pub usize);

/// Thin wrapper around a value of any type. This is primarily what the library takes in and puts out.
#[derive(Clone, Debug, Default)]
pub struct Tile {
    name: String, // TODO: Consider removing this.
    value: TileVisual,
}

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub enum TileVisual {
    Glyph { ch: char },
    Text { text: String },
    Pixel { pixel: Rgba<u8>},

    #[default]
    Default
}

// Compare to a single char
impl PartialEq<char> for TileVisual {
    fn eq(&self, other: &char) -> bool {
        matches!(self, TileVisual::Glyph { ch } if ch == other)
    }
}

// Compare to &str
impl PartialEq<str> for TileVisual {
    fn eq(&self, other: &str) -> bool {
        matches!(self, TileVisual::Text { text } if text == other)
    }
}

// Compare to String (optional; lets you write assert_eq!(visual_data, String::from("wall")))
impl PartialEq<String> for TileVisual {
    fn eq(&self, other: &String) -> bool {
        self == other.as_str()
    }
}

impl PartialEq<Rgba<u8>> for TileVisual {
    fn eq(&self, other: &Rgba<u8>) -> bool {
        matches!(self, TileVisual::Pixel { pixel } if pixel == other)
    }
}


impl fmt::Display for TileVisual {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TileVisual::Glyph { ch } =>
                write!(f, "{}", ch),

            TileVisual::Text { text } =>
                write!(f, "{}", text),

            TileVisual::Pixel { pixel: image } => {
                let rgba = image.channels();
                write!(f, "[RGBA {},{},{},{}]", rgba[0], rgba[1], rgba[2], rgba[3])
            }

            TileVisual::Default => write!(f, "Default"),

            // TileVisual::Image { texture_id, base_uv } =>
            //     write!(f, "[Image tex:{} uv=({}, {}, {}, {})]",
            //            texture_id, base_uv.u0, base_uv.v0, base_uv.u1, base_uv.v1),
        }
    }
}

pub trait ToTileVisual {
    fn to_tile_visual(self) -> TileVisual;
}

impl ToTileVisual for char {
    fn to_tile_visual(self) -> TileVisual {
        TileVisual::Glyph { ch: self }
    }
}

impl TileVisual {
    pub fn as_pixel(&self) -> Option<&Rgba<u8>> {
        match self {
            TileVisual::Pixel { pixel } => Some(pixel),
            _ => None,
        }
    }
}

impl Tile {
    pub fn new(name: String, value: TileVisual) -> Self
    {
        Self {
            name,
            value,
        }
    }

    pub fn from_char(ch: char) -> Self {
        let name = format!("{}", ch);
        Self::new(name, TileVisual::Glyph { ch })
    }

    pub fn from_text(text: String) -> Self {
        Self::new(text.clone(), TileVisual::Text { text })
    }

    pub fn from_pixel(pixel: Rgba<u8>) -> Self {
        Self::new(format!("{:?}", pixel), TileVisual::Pixel { pixel })
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_value(&self) -> &TileVisual {
        &self.value
    }
}

impl PartialEq for Tile {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value // Maybe a bit optimistic - idk if name is relevant yet.
    }
}

impl Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "name: {} | value: {}", self.name, self.value)
    }
}

pub trait ToTile {
    fn to_tile(self) -> Tile;
}

impl ToTile for char {
    fn to_tile(self) -> Tile {
        Tile::from_char(self)
    }
}

impl ToTile for Rgba<u8> {
    fn to_tile(self) -> Tile {
        Tile::from_pixel(self)
    }
}

// mod tests {
//     use crate::tile::Tile;
//
//     #[test]
//     fn value_type() {
//         let str_tile = Tile::new("hello");
//         let char_tile = Tile::new('a');
//         let int_tile = Tile::new(1);
//         let float_tile = Tile::new(1.0);
//         let bool_tile = Tile::new(true);
//         let none_tile = Tile::null();
//
//         assert_eq!(str_tile.value_type, "&str");
//         assert_eq!(char_tile.value_type, "char");
//         assert_eq!(int_tile.value_type, "i32");
//         assert_eq!(float_tile.value_type, "f64");
//         assert_eq!(bool_tile.value_type, "bool");
//         assert_eq!(none_tile.value_type, "");
//     }
//
//     #[test]
//     fn test_is_type() {
//         let tile = Tile::new("hi");
//
//         assert!(tile.is_type::<&str>());
//     }
//
//     #[test]
//     fn test_eq_to() {
//         let tile1 = Tile::new("hi");
//         let tile2 = Tile::new("hi");
//         let tile3 = Tile::new("hello");
//
//         assert!(tile1.eq_to::<&str>(&tile2));
//         assert!(!tile1.eq_to::<&str>(&tile3));
//     }
// }