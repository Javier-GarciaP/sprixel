use super::*;

fn color(key: char, name: &str, hex: &str) -> Color {
    Color {
        key,
        name: name.to_string(),
        hex: hex.to_string(),
    }
}

fn palette() -> Palette {
    Palette {
        name: "main".to_string(),
        colors: vec![
            color('R', "red", "#F04545"),
            color('B', "blue", "#2F6BFF"),
        ],
    }
}

// Checks the "Reserved key" scenario in the core: "." and the space are
// refused with the code `reserved_key`.
#[test]
fn the_dot_and_the_space_are_reserved() {
    let p = palette();
    for key in ['.', ' '] {
        let error = p.check_color(None, &color(key, "new", "#000000")).unwrap_err();
        assert_eq!(error.code, "reserved_key");
    }
}

// Checks that a key must be one printable ASCII character (no accents, no
// control characters), and that case matters (`r` is not `R`).
#[test]
fn a_key_is_printable_ascii_and_case_sensitive() {
    let p = palette();
    for key in ['é', '\n', '\u{7f}', '日'] {
        let error = p.check_color(None, &color(key, "new", "#000000")).unwrap_err();
        assert_eq!(error.code, "bad_key");
    }
    assert!(p.check_color(None, &color('r', "new", "#000000")).is_ok());
    assert!(p.check_color(None, &color('#', "hash", "#000000")).is_ok());
}

// Checks the "Duplicate key" scenario in the core: a key used by another
// color is refused. The color being edited may keep its own key.
#[test]
fn a_key_cannot_be_used_twice() {
    let p = palette();
    let error = p.check_color(None, &color('R', "other", "#000000")).unwrap_err();
    assert_eq!(error.code, "duplicate_key");
    // Editing "R" and leaving its key as it is is fine.
    assert!(p.check_color(Some('R'), &color('R', "red 2", "#D03030")).is_ok());
    // Editing "R" to take the key of "B" is not.
    let error = p.check_color(Some('R'), &color('B', "red", "#D03030")).unwrap_err();
    assert_eq!(error.code, "duplicate_key");
}

// Checks that names are unique ignoring letter case, and not empty.
#[test]
fn a_name_is_unique_and_not_empty() {
    let p = palette();
    let error = p.check_color(None, &color('G', "RED", "#000000")).unwrap_err();
    assert_eq!(error.code, "duplicate_name");
    assert!(p.check_color(Some('R'), &color('R', "Red", "#F04545")).is_ok());
    let error = p.check_color(None, &color('G', "  ", "#000000")).unwrap_err();
    assert_eq!(error.code, "bad_name");
}

// Checks that the value must be `#RRGGBB`.
#[test]
fn a_color_value_is_rrggbb() {
    let p = palette();
    for hex in ["red", "#FFF", "#GG0000", "112233", "#1122334"] {
        let error = p.check_color(None, &color('G', "green", hex)).unwrap_err();
        assert_eq!(error.code, "bad_color", "{hex}");
    }
}

// Checks the limit of 93 colors: a new color on a full palette is
// `palette_full`, but editing a color of a full palette is allowed.
#[test]
fn a_palette_has_at_most_93_colors() {
    let mut p = Palette {
        name: "main".to_string(),
        colors: Vec::new(),
    };
    while let Some(key) = p.next_free_key() {
        let n = p.colors.len();
        p.colors.push(color(key, &format!("c{n}"), "#000000"));
    }
    assert_eq!(p.colors.len(), MAX_COLORS);
    let first = p.colors[0].clone();
    assert_eq!(
        p.check_color(None, &color('A', "one more", "#000000")).unwrap_err().code,
        "palette_full"
    );
    assert!(p.check_color(Some(first.key), &first).is_ok());
}

// Checks that a whole list of colors is checked: this is what the editor
// sends on Done.
#[test]
fn a_list_of_colors_is_checked_as_a_whole() {
    let ok = vec![color('A', "a", "#000000"), color('B', "b", "#FFFFFF")];
    assert!(check_colors(&ok).is_ok());
    let twice = vec![color('A', "a", "#000000"), color('A', "b", "#FFFFFF")];
    assert_eq!(check_colors(&twice).unwrap_err().code, "duplicate_key");
    let same_name = vec![color('A', "a", "#000000"), color('B', "A", "#FFFFFF")];
    assert_eq!(check_colors(&same_name).unwrap_err().code, "duplicate_name");
    // Only 93 keys exist, so the 94th repeats one; the limit is reported first.
    let keys: Vec<char> = ('!'..='~').filter(|c| *c != '.').collect();
    let too_many: Vec<Color> = (0..94)
        .map(|i| color(keys[i % 93], &format!("c{i}"), "#000000"))
        .collect();
    assert_eq!(check_colors(&too_many).unwrap_err().code, "palette_full");
}
