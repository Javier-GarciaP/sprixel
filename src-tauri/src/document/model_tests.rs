use super::*;

fn color(key: char) -> Color {
    Color {
        key,
        name: format!("color {key}"),
        hex: "#112233".to_string(),
    }
}

fn palette(keys: &str) -> Palette {
    Palette {
        name: "dungeon16".to_string(),
        colors: keys.chars().map(color).collect(),
    }
}

// Checks that a new sprite has one layer, one frame and all pixels transparent.
#[test]
fn new_sprite_has_one_layer_and_one_frame() {
    let sprite = Sprite::new("goblin", 8, 4, "dungeon16").unwrap();
    assert_eq!(sprite.layers.len(), 1);
    assert_eq!(sprite.frame_count(), 1);
    assert_eq!(sprite.frame_durations.len(), 1);
    assert_eq!(sprite.layers[0].frames[0].pixels, vec![b'.'; 32]);
}

// Checks the size limits: 1 to 512 per side, otherwise `too_big` or `bad_size`.
#[test]
fn sprite_size_limits() {
    assert_eq!(Sprite::new("a", 600, 20, "p").unwrap_err().code, "too_big");
    assert_eq!(Sprite::new("a", 20, 513, "p").unwrap_err().code, "too_big");
    assert_eq!(Sprite::new("a", 0, 20, "p").unwrap_err().code, "bad_size");
    assert_eq!(Sprite::new("a", 20, 0, "p").unwrap_err().code, "bad_size");
    assert!(Sprite::new("a", 1, 1, "p").is_ok());
    assert!(Sprite::new("a", 512, 512, "p").is_ok());
}

// Checks that a valid sprite passes validation against its palette.
#[test]
fn valid_sprite_passes() {
    let sprite = Sprite::new("a", 2, 2, "dungeon16").unwrap();
    assert!(sprite.validate(&palette("NP")).is_ok());
}

// Checks that a pixel with a key missing from the palette is rejected.
#[test]
fn unknown_pixel_key_is_rejected() {
    let mut sprite = Sprite::new("a", 2, 2, "dungeon16").unwrap();
    sprite.layers[0].frames[0].pixels[3] = b'X';
    assert_eq!(
        sprite.validate(&palette("NP")).unwrap_err().code,
        "unknown_key"
    );
}

// Checks that a frame with a different size than the sprite is rejected.
#[test]
fn frame_with_wrong_size_is_rejected() {
    let mut sprite = Sprite::new("a", 2, 2, "dungeon16").unwrap();
    sprite.layers[0].frames[0].pixels.pop();
    assert_eq!(
        sprite.validate(&palette("N")).unwrap_err().code,
        "bad_sprite"
    );
}

// Checks that a sprite needs at least one layer and one frame.
#[test]
fn sprite_needs_a_layer_and_a_frame() {
    let mut no_layers = Sprite::new("a", 2, 2, "dungeon16").unwrap();
    no_layers.layers.clear();
    assert_eq!(
        no_layers.validate(&palette("N")).unwrap_err().code,
        "bad_sprite"
    );

    let mut no_frames = Sprite::new("a", 2, 2, "dungeon16").unwrap();
    no_frames.layers[0].frames.clear();
    no_frames.frame_durations.clear();
    assert_eq!(
        no_frames.validate(&palette("N")).unwrap_err().code,
        "bad_sprite"
    );
}

// Checks that every layer has one frame per duration.
#[test]
fn layers_and_durations_must_match_the_frame_count() {
    let mut sprite = Sprite::new("a", 2, 2, "dungeon16").unwrap();
    sprite.frame_durations.push(100);
    assert_eq!(
        sprite.validate(&palette("N")).unwrap_err().code,
        "bad_sprite"
    );
}

fn tag(from: u32, to: u32, fps: u32) -> Tag {
    Tag {
        name: "walk".to_string(),
        from,
        to,
        fps,
        mode: TagMode::Loop,
    }
}

// Checks that a tag inside the frames is valid (frames are numbered from 1).
#[test]
fn tag_inside_the_frames_is_valid() {
    assert!(tag(1, 4, 12).validate(4).is_ok());
    assert!(tag(2, 2, 12).validate(4).is_ok());
}

// Checks that a tag with frames 3 to 8 on a sprite with 4 frames is `bad_range`.
#[test]
fn tag_range_outside_the_frames_is_rejected() {
    assert_eq!(tag(3, 8, 12).validate(4).unwrap_err().code, "bad_range");
    assert_eq!(tag(0, 2, 12).validate(4).unwrap_err().code, "bad_range");
}

// Checks that a tag whose start is after its end is `bad_range`.
#[test]
fn tag_start_after_end_is_rejected() {
    assert_eq!(tag(3, 2, 12).validate(4).unwrap_err().code, "bad_range");
}

// Checks the speed limits of a tag: 1 to 60 fps.
#[test]
fn tag_fps_limits() {
    assert_eq!(tag(1, 2, 0).validate(4).unwrap_err().code, "bad_fps");
    assert_eq!(tag(1, 2, 61).validate(4).unwrap_err().code, "bad_fps");
    assert!(tag(1, 2, 1).validate(4).is_ok());
    assert!(tag(1, 2, 60).validate(4).is_ok());
}

// Checks that tag names must be unique in a sprite, ignoring case.
#[test]
fn tag_names_are_unique_in_a_sprite() {
    let mut sprite = Sprite::new("a", 2, 2, "dungeon16").unwrap();
    sprite.tags.push(tag(1, 1, 12));
    let mut other = tag(1, 1, 12);
    other.name = "WALK".to_string();
    sprite.tags.push(other);
    assert_eq!(
        sprite.validate(&palette("N")).unwrap_err().code,
        "tag_exists"
    );
}

// Checks that tag modes are read from and written to their file names.
#[test]
fn tag_modes_round_trip_through_text() {
    for text in ["loop", "once", "pingpong"] {
        assert_eq!(TagMode::parse(text).unwrap().as_str(), text);
    }
    assert!(TagMode::parse("bounce").is_none());
}

// Checks that a palette holds up to 93 colors and not one more.
#[test]
fn palette_allows_up_to_93_colors() {
    let keys: String = ('!'..='~').filter(|c| *c != '.').collect();
    assert_eq!(keys.len(), 93);
    assert!(palette(&keys).validate().is_ok());
    let mut too_many = palette(&keys);
    too_many.colors.push(color('\u{7f}'));
    assert_eq!(too_many.validate().unwrap_err().code, "bad_palette");
}

// Checks that `.`, spaces, control and non-ASCII characters cannot be keys.
#[test]
fn palette_keys_must_be_printable_ascii_but_not_dot() {
    for key in ['.', ' ', '\n', 'é'] {
        assert_eq!(
            palette(&key.to_string()).validate().unwrap_err().code,
            "bad_palette"
        );
    }
}

// Checks that keys are case sensitive: `a` and `A` differ, but `a` twice is not allowed.
#[test]
fn palette_keys_are_unique_and_case_sensitive() {
    assert!(palette("aA").validate().is_ok());
    assert_eq!(palette("aa").validate().unwrap_err().code, "bad_palette");
}

// Checks that a color hex must look like #RRGGBB.
#[test]
fn palette_colors_need_a_valid_hex() {
    for hex in ["112233", "#12345", "#GGGGGG", "#1122334"] {
        let mut p = palette("N");
        p.colors[0].hex = hex.to_string();
        assert_eq!(p.validate().unwrap_err().code, "bad_palette", "{hex}");
    }
}

// Checks that `.` is always an accepted pixel, and so are the palette keys.
#[test]
fn palette_accepts_transparent_and_its_own_keys() {
    let p = palette("NP");
    assert!(p.accepts('.'));
    assert!(p.accepts('N'));
    assert!(!p.accepts('n'));
}

// Checks that a new project is empty, at revision 0, and finds palettes ignoring case.
#[test]
fn new_project_is_empty() {
    let mut project = Project::new("Dungeon Kit");
    assert_eq!(project.revision, 0);
    assert!(project.root.sprites.is_empty());
    project.palettes.push(palette("N"));
    assert!(project.palette("DUNGEON16").is_some());
    assert!(project.palette("other").is_none());
}
