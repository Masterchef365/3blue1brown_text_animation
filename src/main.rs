use anyhow::{Context, Result};
use owned_ttf_parser::{self as ttf, OutlineBuilder, GlyphId};
use harfbuzz_rs::{self as hb, Face as HbFace, UnicodeBuffer};

struct Outliner;

impl OutlineBuilder for Outliner {
    fn move_to(&mut self, x: f32, y: f32) {
        println!("move_to (x: {}, y: {})", x, y);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        println!("line_to (x: {}, y: {})", x, y);
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        println!("quad_to (x1: {}, y1: {} x: {}, y: {})", x1, y1, x, y);
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        println!(
            "curve_to (x2: {}, y2: {}, x1: {}, y1: {} x: {}, y: {})",
            x2, y2, x1, y1, x, y
        );
    }

    fn close(&mut self) {
        println!("close()");
    }
}

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let ttf_path = args.next().context("Requires TTF path")?;
    let text = args.next().context("Requires text")?;
    let font_index = 0;

    let ttf_data = std::fs::read(ttf_path).context("Failed to read TTF")?;
    let ttf_face = ttf::Face::from_slice(&ttf_data, font_index).context("Failed to parse TTF")?;

    let unicode = UnicodeBuffer::new().add_str(&text);
    let hb_face = hb::Face::new(&ttf_data, font_index);
    let hb_font = hb::Font::new(hb_face);
    let shape = hb::shape(&hb_font, unicode, &[]);
    let positions = shape.get_glyph_positions();
    let infos = shape.get_glyph_infos();

    for (position, info) in positions.iter().zip(infos) {
        let mut outliner = Outliner;
        let rect = ttf_face.outline_glyph(GlyphId(info.codepoint as u16), &mut outliner);
        dbg!(rect);
        dbg!(position);
    }
    Ok(())
}
