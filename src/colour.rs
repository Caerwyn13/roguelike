use serde::{Serialize, Deserialize};
use tcod::colors::Color;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SerializableColour {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl From<Color> for SerializableColour {
    fn from(color: Color) -> Self {
        SerializableColour { r: color.r, g: color.g, b: color.b }
    }
}

impl From<SerializableColour> for Color {
    fn from(scolor: SerializableColour) -> Self {
        Color::new(scolor.r, scolor.g, scolor.b)
    }
}
