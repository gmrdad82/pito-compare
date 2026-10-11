use std::io::Write;
use std::path::Path;

use crate::{Error, Result};

pub type Color = [u8; 3];

pub const BG: Color = [16, 19, 25];
pub const TEXT: Color = [232, 235, 240];
pub const DIM: Color = [138, 146, 160];
pub const GRID: Color = [44, 50, 62];
pub const SEP: Color = [70, 78, 92];
pub const OURS: Color = [220, 110, 34];
pub const THEIRS: Color = [74, 142, 230];
pub const FASTER: Color = [74, 201, 125];
pub const SLOWER: Color = [235, 88, 88];

const GLYPHS: &[(char, [u8; 7])] = &[
    ('A', [14, 17, 17, 31, 17, 17, 17]),
    ('B', [30, 17, 17, 30, 17, 17, 30]),
    ('C', [14, 17, 16, 16, 16, 17, 14]),
    ('D', [30, 17, 17, 17, 17, 17, 30]),
    ('E', [31, 16, 16, 30, 16, 16, 31]),
    ('F', [31, 16, 16, 30, 16, 16, 16]),
    ('G', [14, 17, 16, 23, 17, 17, 15]),
    ('H', [17, 17, 17, 31, 17, 17, 17]),
    ('I', [14, 4, 4, 4, 4, 4, 14]),
    ('J', [7, 2, 2, 2, 2, 18, 12]),
    ('K', [17, 18, 20, 24, 20, 18, 17]),
    ('L', [16, 16, 16, 16, 16, 16, 31]),
    ('M', [17, 27, 21, 21, 17, 17, 17]),
    ('N', [17, 25, 21, 19, 17, 17, 17]),
    ('O', [14, 17, 17, 17, 17, 17, 14]),
    ('P', [30, 17, 17, 30, 16, 16, 16]),
    ('Q', [14, 17, 17, 17, 21, 18, 13]),
    ('R', [30, 17, 17, 30, 20, 18, 17]),
    ('S', [15, 16, 16, 14, 1, 1, 30]),
    ('T', [31, 4, 4, 4, 4, 4, 4]),
    ('U', [17, 17, 17, 17, 17, 17, 14]),
    ('V', [17, 17, 17, 17, 17, 10, 4]),
    ('W', [17, 17, 17, 21, 21, 21, 10]),
    ('X', [17, 17, 10, 4, 10, 17, 17]),
    ('Y', [17, 17, 10, 4, 4, 4, 4]),
    ('Z', [31, 1, 2, 4, 8, 16, 31]),
    ('0', [14, 17, 19, 21, 25, 17, 14]),
    ('1', [4, 12, 4, 4, 4, 4, 14]),
    ('2', [14, 17, 1, 2, 4, 8, 31]),
    ('3', [30, 1, 1, 14, 1, 1, 30]),
    ('4', [2, 6, 10, 18, 31, 2, 2]),
    ('5', [31, 16, 30, 1, 1, 17, 14]),
    ('6', [6, 8, 16, 30, 17, 17, 14]),
    ('7', [31, 1, 2, 4, 8, 8, 8]),
    ('8', [14, 17, 17, 14, 17, 17, 14]),
    ('9', [14, 17, 17, 15, 1, 2, 12]),
    ('.', [0, 0, 0, 0, 0, 12, 12]),
    (',', [0, 0, 0, 0, 12, 4, 8]),
    (':', [0, 12, 12, 0, 12, 12, 0]),
    ('-', [0, 0, 0, 31, 0, 0, 0]),
    ('+', [0, 4, 4, 31, 4, 4, 0]),
    ('/', [1, 1, 2, 4, 8, 16, 16]),
    ('(', [2, 4, 8, 8, 8, 4, 2]),
    (')', [8, 4, 2, 2, 2, 4, 8]),
    ('%', [25, 26, 2, 4, 8, 11, 19]),
    ('=', [0, 0, 31, 0, 31, 0, 0]),
    ('<', [2, 4, 8, 16, 8, 4, 2]),
    ('>', [8, 4, 2, 1, 2, 4, 8]),
    ('\'', [4, 4, 8, 0, 0, 0, 0]),
    ('_', [0, 0, 0, 0, 0, 0, 31]),
    ('×', [0, 17, 10, 4, 10, 17, 0]),
    ('*', [0, 21, 14, 31, 14, 21, 0]),
    ('†', [4, 14, 4, 4, 4, 4, 0]),
    (';', [0, 12, 12, 0, 12, 4, 8]),
    ('|', [4, 4, 4, 4, 4, 4, 4]),
    (' ', [0, 0, 0, 0, 0, 0, 0]),
];

fn find(c: char) -> Option<[u8; 7]> {
    let c = c.to_ascii_uppercase();
    GLYPHS.iter().find(|(g, _)| *g == c).map(|(_, rows)| *rows)
}

pub fn has_glyph(c: char) -> bool {
    find(c).is_some()
}

fn glyph(c: char) -> [u8; 7] {
    find(c).unwrap_or([0; 7])
}

pub struct Canvas {
    width: usize,
    height: usize,
    px: Vec<u8>,
}

impl Canvas {
    pub fn new(width: usize, height: usize) -> Self {
        let mut px = Vec::with_capacity(width * height * 3);
        for _ in 0..width * height {
            px.extend_from_slice(&BG);
        }
        Self { width, height, px }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn pixel(&self, x: usize, y: usize) -> Option<Color> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let at = (y * self.width + x) * 3;
        Some([self.px[at], self.px[at + 1], self.px[at + 2]])
    }

    pub fn rect(&mut self, x: i64, y: i64, w: i64, h: i64, c: Color) {
        let x0 = x.clamp(0, self.width as i64) as usize;
        let x1 = (x + w).clamp(0, self.width as i64) as usize;
        let y0 = y.clamp(0, self.height as i64) as usize;
        let y1 = (y + h).clamp(0, self.height as i64) as usize;
        for row in y0..y1 {
            for col in x0..x1 {
                let at = (row * self.width + col) * 3;
                self.px[at..at + 3].copy_from_slice(&c);
            }
        }
    }

    pub fn text_width(text: &str, scale: usize) -> i64 {
        (text.chars().count() * 6 * scale) as i64 - scale as i64
    }

    pub fn text(&mut self, x: i64, y: i64, text: &str, scale: usize, c: Color) {
        let s = scale as i64;
        let mut pen = x;
        for ch in text.chars() {
            let rows = glyph(ch);
            for (r, bits) in rows.iter().enumerate() {
                for col in 0..5 {
                    if bits >> (4 - col) & 1 == 1 {
                        self.rect(pen + col as i64 * s, y + r as i64 * s, s, s, c);
                    }
                }
            }
            pen += 6 * s;
        }
    }

    pub fn write_png<W: Write>(&self, out: W) -> Result<()> {
        let mut enc = png::Encoder::new(out, self.width as u32, self.height as u32);
        enc.set_color(png::ColorType::Rgb);
        enc.set_depth(png::BitDepth::Eight);
        enc.set_compression(png::Compression::High);
        let mut w = enc
            .write_header()
            .map_err(|e| Error::new(format!("cannot write the PNG: {e}")))?;
        w.write_image_data(&self.px)
            .map_err(|e| Error::new(format!("cannot write the PNG: {e}")))?;
        w.finish()
            .map_err(|e| Error::new(format!("cannot write the PNG: {e}")))
    }

    pub fn png(&self) -> Result<Vec<u8>> {
        let mut out = Vec::new();
        self.write_png(&mut out)?;
        Ok(out)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let bytes = self.png()?;
        if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
            std::fs::create_dir_all(dir)
                .map_err(|e| Error::new(format!("cannot create {}: {e}", dir.display())))?;
        }
        std::fs::write(path, bytes)
            .map_err(|e| Error::new(format!("cannot write {}: {e}", path.display())))
    }
}
