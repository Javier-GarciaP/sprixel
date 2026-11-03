/// How many frames a row of the sheet holds at most.
pub const FRAMES_PER_ROW: usize = 8;

/// Where a frame goes in the sheet, in pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Placement {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

/// The size of the sheet and the place of every frame, in the input order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Layout {
    pub width: u32,
    pub height: u32,
    pub placements: Vec<Placement>,
}

/// Puts frames in rows, left to right, `per_row` frames in each row, with
/// `spacing` pixels between frames and between rows. A row is as tall as its
/// tallest frame. No spacing is added at the edges of the sheet.
pub fn pack_rows(sizes: &[(u32, u32)], spacing: u32, per_row: usize) -> Layout {
    let mut placements = Vec::with_capacity(sizes.len());
    let (mut width, mut y) = (0, 0);
    for row in sizes.chunks(per_row.max(1)) {
        let mut x = 0;
        let mut row_height = 0;
        for &(w, h) in row {
            placements.push(Placement { x, y, w, h });
            x += w + spacing;
            row_height = row_height.max(h);
        }
        width = width.max(x - spacing);
        y += row_height + spacing;
    }
    let height = if sizes.is_empty() { 0 } else { y - spacing };
    Layout { width, height, placements }
}
