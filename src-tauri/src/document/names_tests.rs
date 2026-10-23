use super::names::{same_name, validate_name};

// Checks that ordinary names, including accents and spaces inside, are valid.
#[test]
fn normal_names_are_valid() {
    for name in ["goblin", "Goblin 2", "Árbol", "ñandú", "slime_01", "a.b", "日本語"] {
        assert!(validate_name(name).is_ok(), "{name}");
    }
}

// Checks that an empty name or a name of only spaces is rejected.
#[test]
fn empty_names_are_rejected() {
    for name in ["", " ", "   "] {
        assert_eq!(validate_name(name).unwrap_err().code, "invalid_name");
    }
}

// Checks the characters that are invalid in file names on Linux, Windows or macOS.
#[test]
fn invalid_characters_are_rejected() {
    for name in [
        "a/b", "a\\b", "a:b", "a*b", "a?b", "a\"b", "a<b", "a>b", "a|b", "a\0b", "a\nb", "a\tb",
    ] {
        assert_eq!(validate_name(name).unwrap_err().code, "invalid_name", "{name:?}");
    }
}

// Checks the reserved Windows device names, in any case and with an extension.
#[test]
fn reserved_windows_names_are_rejected() {
    for name in [
        "CON", "con", "Prn", "AUX", "NUL", "COM1", "com9", "LPT1", "lpt9", "CON.txt", "nul.sprite",
    ] {
        assert_eq!(validate_name(name).unwrap_err().code, "invalid_name", "{name}");
    }
    // Names that only start like a reserved one are fine.
    for name in ["CONSOLE", "COM10", "COM0", "LPT", "aux2"] {
        assert!(validate_name(name).is_ok(), "{name}");
    }
}

// Checks that names ending in a dot or a space (not allowed on Windows) and `.` / `..` are rejected.
#[test]
fn trailing_dots_and_spaces_are_rejected() {
    for name in ["goblin.", "goblin ", " goblin", ".", ".."] {
        assert_eq!(validate_name(name).unwrap_err().code, "invalid_name", "{name:?}");
    }
}

// Checks that a very long name is rejected, so the file name stays short enough.
#[test]
fn very_long_names_are_rejected() {
    assert!(validate_name(&"a".repeat(100)).is_ok());
    assert_eq!(
        validate_name(&"a".repeat(201)).unwrap_err().code,
        "invalid_name"
    );
}

// Checks that names are compared ignoring upper and lower case, also with accents.
#[test]
fn names_compare_ignoring_case_and_with_accents() {
    assert!(same_name("Goblin", "goblin"));
    assert!(same_name("ÁRBOL", "árbol"));
    assert!(same_name("Ñandú", "ñANDÚ"));
    assert!(!same_name("arbol", "árbol"));
    assert!(!same_name("goblin", "goblin2"));
}
