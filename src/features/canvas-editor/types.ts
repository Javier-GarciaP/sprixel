// A color of the sprite palette: one printable character and a #RRGGBB value.
export type PaletteColor = { key: string; hex: string };

// A rectangle in sprite pixels (or in screen pixels, where the name says so).
export type Rect = { x: number; y: number; width: number; height: number };

// The pixels of a frame are kept as one byte per pixel: the character code of
// the palette key. This is the same data the core stores, so nothing has to
// be converted when a stroke is sent. `.` (transparent) is 46.
export const TRANSPARENT_CODE = ".".charCodeAt(0);
