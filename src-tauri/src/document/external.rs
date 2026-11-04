use super::error::DocError;
use super::model::{Folder, Palette, Project, Sprite};
use super::names::same_name;
use super::operation::SpriteRef;

// Changes that come from files edited outside the app (for example after a
// `git pull`). They are not operations: they have no author and no inverse,
// so the caller clears the history. The revision still grows.
impl Project {
    /// Replaces the sprite with the same name, or adds it. Missing folders
    /// are created.
    pub fn put_sprite(&mut self, folder: &[String], sprite: Sprite) {
        let mut target = &mut self.root;
        for name in folder {
            let index = match target.folders.iter().position(|f| same_name(&f.name, name)) {
                Some(index) => index,
                None => {
                    target.folders.push(Folder::new(name));
                    target.folders.len() - 1
                }
            };
            target = &mut target.folders[index];
        }
        match target.sprites.iter().position(|s| same_name(&s.name, &sprite.name)) {
            Some(index) => target.sprites[index] = sprite,
            None => target.sprites.push(sprite),
        }
        self.revision += 1;
    }

    /// Removes a sprite. Returns false if it was not there.
    pub fn remove_sprite(&mut self, sprite: &SpriteRef) -> bool {
        let Some((folder, index)) = self.locate_sprite(sprite) else {
            return false;
        };
        self.folder_at_mut(&folder).sprites.remove(index);
        self.revision += 1;
        true
    }

    /// Replaces the palette with the same name, or adds it. If a sprite that
    /// uses the palette would break, nothing changes.
    pub fn put_palette(&mut self, palette: Palette) -> Result<(), DocError> {
        palette.validate()?;
        for sprite in self.all_sprites().filter(|s| same_name(&s.palette, &palette.name)) {
            sprite.validate(&palette)?;
        }
        match self.palettes.iter().position(|p| same_name(&p.name, &palette.name)) {
            Some(index) => self.palettes[index] = palette,
            None => self.palettes.push(palette),
        }
        self.revision += 1;
        Ok(())
    }

    /// Removes a palette. Returns false if it was not there, and an error if
    /// a sprite uses it.
    pub fn remove_palette(&mut self, name: &str) -> Result<bool, DocError> {
        let Some(index) = self.palettes.iter().position(|p| same_name(&p.name, name)) else {
            return Ok(false);
        };
        if self.all_sprites().any(|s| same_name(&s.palette, name)) {
            return Err(DocError::new(
                "palette_in_use",
                format!("palette {name:?} is used by a sprite"),
            ));
        }
        self.palettes.remove(index);
        self.revision += 1;
        Ok(true)
    }
}
