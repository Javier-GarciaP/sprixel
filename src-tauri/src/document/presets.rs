//! The built-in palettes a person can start from. The names are generic on
//! purpose: no trademarks and no names of known palettes.

use super::model::{Color, Palette};

pub struct Preset {
    /// Stable id the UI sends back (`pocket-rpg-16`).
    pub id: &'static str,
    /// The name shown to people, and the default name of the new palette.
    pub name: &'static str,
    /// `(key, name, hex)` for each color.
    colors: &'static [(char, &'static str, &'static str)],
}

pub const PRESETS: &[Preset] = &[
    Preset {
        id: "pocket-rpg-16",
        name: "Pocket RPG 16",
        colors: &[
            ('K', "ink", "#1B1B2F"),
            ('W', "snow", "#F4F1E8"),
            ('S', "stone", "#8A8A96"),
            ('D', "slate", "#3B3B46"),
            ('R', "red", "#E04646"),
            ('O', "orange", "#F08A3C"),
            ('Y', "gold", "#F5C542"),
            ('G', "grass", "#4ADE80"),
            ('F', "forest", "#2E8B4A"),
            ('T', "teal", "#2BB3A8"),
            ('B', "blue", "#2F6BFF"),
            ('N', "navy", "#243B8C"),
            ('P', "purple", "#8B5CD6"),
            ('M', "pink", "#E879A8"),
            ('H', "skin", "#FFCC99"),
            ('C', "brown", "#8B5A2B"),
        ],
    },
    Preset {
        id: "farm-32",
        name: "Farm 32",
        colors: &[
            ('A', "night", "#10121C"),
            ('B', "soot", "#2A2D3A"),
            ('C', "slate", "#4A4E5E"),
            ('D', "ash", "#7B8090"),
            ('E', "mist", "#B4B9C6"),
            ('F', "cloud", "#E8ECF2"),
            ('G', "milk", "#FFF8E7"),
            ('H', "wheat", "#F2D98A"),
            ('I', "straw", "#E0B84F"),
            ('J', "corn", "#F5C542"),
            ('K', "pumpkin", "#E8862E"),
            ('L', "carrot", "#D96A1E"),
            ('M', "tomato", "#D83A34"),
            ('N', "berry", "#A12A4B"),
            ('O', "plum", "#6B2E6B"),
            ('P', "lavender", "#9B7ED8"),
            ('Q', "sky", "#7EC8F0"),
            ('R', "lake", "#3A8FD6"),
            ('S', "deep", "#1F4E9C"),
            ('T', "pond", "#2E7D8C"),
            ('U', "mint", "#9BE5A8"),
            ('V', "sprout", "#5CC95F"),
            ('W', "leaf", "#34A04A"),
            ('X', "pine", "#1E6B3A"),
            ('Y', "moss", "#6B7F2E"),
            ('Z', "clay", "#C98A5A"),
            ('a', "soil", "#8B5A2B"),
            ('b', "earth", "#5C3A1E"),
            ('c', "bark", "#3B2414"),
            ('d', "sand", "#E6C79C"),
            ('e', "rose", "#F2A7B0"),
            ('f', "cream", "#FBE9C6"),
        ],
    },
];

impl Preset {
    pub fn find(id: &str) -> Option<&'static Preset> {
        PRESETS.iter().find(|p| p.id == id)
    }

    pub fn colors(&self) -> Vec<Color> {
        self.colors
            .iter()
            .map(|(key, name, hex)| Color {
                key: *key,
                name: name.to_string(),
                hex: hex.to_string(),
            })
            .collect()
    }

    /// A palette with these colors and the given name.
    pub fn palette(&self, name: &str) -> Palette {
        Palette {
            name: name.to_string(),
            colors: self.colors(),
        }
    }
}
