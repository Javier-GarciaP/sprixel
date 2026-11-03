use super::*;

// Checks that "Pocket RPG 16" has 16 colors and "Farm 32" has 32, and that
// every color of every preset passes the same rules as a color typed by hand
// (key, unique name, #RRGGBB), so a new palette from a preset is always valid.
#[test]
fn presets_have_the_right_number_of_valid_colors() {
    let count = |id: &str| Preset::find(id).unwrap().colors().len();
    assert_eq!(count("pocket-rpg-16"), 16);
    assert_eq!(count("farm-32"), 32);
    for preset in PRESETS {
        let palette = preset.palette(preset.name);
        palette.validate().unwrap();
        check_colors(&palette.colors).unwrap();
    }
}

// Checks that each preset has unique keys and also unique hex values (two
// colors with the same value would be a mistake in the data).
#[test]
fn preset_keys_and_values_are_unique() {
    for preset in PRESETS {
        let colors = preset.colors();
        for (i, a) in colors.iter().enumerate() {
            for b in &colors[i + 1..] {
                assert_ne!(a.key, b.key, "{}", preset.id);
                assert_ne!(a.hex, b.hex, "{}", preset.id);
            }
        }
    }
}

// Checks that the names are generic: none of the names of the presets or of
// their colors contains a trademark or the name of a known palette.
#[test]
fn preset_names_have_no_trademarks() {
    let banned = [
        "pico", "nes", "gameboy", "game boy", "nintendo", "sega", "atari", "commodore", "c64",
        "endesga", "lospec", "sweetie", "apollo", "resurrect", "db16", "db32", "stardew", "zelda",
        "pokemon", "pokémon", "mario",
    ];
    for preset in PRESETS {
        let mut words = vec![preset.name.to_lowercase(), preset.id.to_lowercase()];
        words.extend(preset.colors().iter().map(|c| c.name.to_lowercase()));
        for word in words {
            for ban in banned {
                assert!(!word.contains(ban), "{word:?} contains {ban:?}");
            }
        }
    }
}

// Checks that an unknown id is not a preset.
#[test]
fn an_unknown_preset_is_none() {
    assert!(Preset::find("nope").is_none());
}
