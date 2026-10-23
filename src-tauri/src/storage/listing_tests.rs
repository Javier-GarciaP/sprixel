use super::session_tests::dungeon16;
use super::*;
use crate::document::{Sprite, SpriteRef};
use std::fs;
use std::path::Path;
use std::time::Instant;

/// Writes a project with `folders * per_folder` sprites of 32x32 pixels, each
/// with a pattern of colors, straight to disk. Returns nothing: it only builds files.
fn write_big_project(root: &Path, folders: usize, per_folder: usize) {
    write_manifest(
        &DiskWriter,
        root,
        &Manifest {
            name: "Big".to_string(),
            active_palette: None,
        },
    )
    .unwrap();
    write_palette(&DiskWriter, root, &dungeon16()).unwrap();
    for f in 0..folders {
        let folder = format!("Folder {f}");
        for s in 0..per_folder {
            let name = format!("sprite {s:03}");
            let mut sprite = Sprite::new(&name, 32, 32, "dungeon16").unwrap();
            for (i, pixel) in sprite.layers[0].frames[0].pixels.iter_mut().enumerate() {
                *pixel = b"NPAR."[(i + s) % 5];
            }
            let sprite_ref = SpriteRef::new(&[folder.as_str()], &name);
            write_sprite(&DiskWriter, root, &sprite_ref, &sprite).unwrap();
        }
    }
}

// Checks that the listing has the name, size, palette and frame count of each sprite
// and the folders and palettes, in a steady order.
#[test]
fn the_listing_has_the_project_browser_data() {
    let parent = tempfile::tempdir().unwrap();
    write_big_project(parent.path(), 2, 3);

    let listing = list_project(parent.path()).unwrap();

    assert_eq!(listing.name, "Big");
    assert_eq!(listing.palettes, ["dungeon16"]);
    assert_eq!(listing.folders, [vec!["Folder 0"], vec!["Folder 1"]]);
    assert_eq!(listing.sprites.len(), 6);
    let first = &listing.sprites[0];
    assert_eq!(first.folder, ["Folder 0"]);
    assert_eq!(first.name, "sprite 000");
    assert_eq!((first.width, first.height, first.frames), (32, 32, 1));
    assert_eq!(first.palette, "dungeon16");
    assert!(listing.damaged.is_empty());
}

// Checks that the listing does not load pixel data: a sprite with wrong rows still
// lists with its size (the pixels are checked only when the sprite is opened).
#[test]
fn the_listing_does_not_read_the_pixels() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path();
    write_big_project(root, 1, 1);
    let path = root.join("sprites/Folder 0/sprite 000.sprite.json");
    let text = fs::read_to_string(&path).unwrap();
    fs::write(&path, text.replacen("\"NPAR.NPAR", "\"X", 1)).unwrap();

    assert_eq!(list_project(root).unwrap().sprites.len(), 1);
    assert_eq!(Session::open(root).unwrap().damaged().len(), 1);
}

// Checks that a file with invalid JSON is listed as damaged with its path, and the rest still lists.
#[test]
fn the_listing_reports_damaged_files() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path();
    write_big_project(root, 1, 2);
    let broken = root.join("sprites/Folder 0/sprite 000.sprite.json");
    fs::write(&broken, "{ nope").unwrap();

    let listing = list_project(root).unwrap();

    assert_eq!(listing.sprites.len(), 1);
    assert_eq!(listing.damaged.len(), 1);
    assert_eq!(listing.damaged[0].path, broken);
}

// Checks the spec scenario: a project with 500 sprites of 32x32 lists all of them in
// 2 seconds or less. It also prints the time (run with `-- --nocapture`), and the
// time to open the whole project with all its pixels, for comparison.
#[test]
fn a_project_with_500_sprites_lists_in_two_seconds() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path();
    write_big_project(root, 10, 50);

    let start = Instant::now();
    let listing = list_project(root).unwrap();
    let listing_time = start.elapsed();

    let start = Instant::now();
    let session = Session::open(root).unwrap();
    let open_time = start.elapsed();

    println!("500 sprites 32x32: listing {listing_time:?}, full open {open_time:?}");
    assert_eq!(listing.sprites.len(), 500);
    assert_eq!(session.project.root.folders.len(), 10);
    assert!(
        listing_time.as_secs_f64() <= 2.0,
        "listing took {listing_time:?}, the limit is 2 seconds"
    );
}
