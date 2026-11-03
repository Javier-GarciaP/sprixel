use super::ramp::ramp;

// Checks the exact tones of a red ramp with 3 steps (the example of the spec).
#[test]
fn three_tones_of_red() {
    assert_eq!(ramp("#FF0000", 3, 0.0).unwrap(), ["#590000", "#FF0000", "#FFA6A6"]);
}

// Checks that the base is always in the ramp and that the size is right for
// every allowed number of steps (more dark tones than light on even steps).
#[test]
fn the_base_is_inside_and_the_size_is_right() {
    for steps in 3..=9 {
        let tones = ramp("#3366CC", steps, 0.0).unwrap();
        assert_eq!(tones.len(), steps);
        assert_eq!(tones[steps - 1 - (steps - 1) / 2], "#3366CC", "steps {steps}");
    }
}

// Checks the exact tones of a blue ramp with 5 steps (2 dark, base, 2 light).
#[test]
fn five_tones_of_blue() {
    assert_eq!(ramp("#3366CC", 5, 0.0).unwrap(), ["#122447", "#22458A", "#3366CC", "#7598DD", "#B8C9ED"]);
}

// Checks the hue shift: the light end turns forward, the dark end backward.
#[test]
fn hue_shift_turns_the_ends() {
    let plain = ramp("#FF0000", 3, 0.0).unwrap();
    let shifted = ramp("#FF0000", 3, 30.0).unwrap();
    assert_eq!(shifted[1], "#FF0000", "the base does not move");
    assert_ne!(shifted[0], plain[0]);
    assert_eq!(shifted[2], "#FFD2A6", "red turned 30 degrees toward yellow");
}

// Checks that a gray stays gray, and that the steps and hue limits are checked.
#[test]
fn gray_and_bad_arguments() {
    for tone in ramp("#808080", 3, 40.0).unwrap() {
        assert_eq!(tone[1..3], tone[3..5]);
        assert_eq!(tone[3..5], tone[5..7]);
    }
    for steps in [2, 10] {
        assert_eq!(ramp("#FF0000", steps, 0.0).unwrap_err().code, "bad_argument");
    }
    assert_eq!(ramp("#FF0000", 3, 61.0).unwrap_err().code, "bad_argument");
    assert_eq!(ramp("red", 3, 0.0).unwrap_err().code, "bad_argument");
}
