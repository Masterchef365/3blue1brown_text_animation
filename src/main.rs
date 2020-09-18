use anyhow::{Context, Result};
use owned_ttf_parser::{Face, OutlineBuilder};

//TODO: Use harfbuzz to figure out texture locations

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
    let ttf_data = std::fs::read(ttf_path).context("Failed to read TTF")?;
    let face = Face::from_slice(&ttf_data, 0).context("Failed to parse TTF")?;
    for character in text.chars() {
        let mut outliner = Outliner;
        let glyph_id = face
            .glyph_index(character)
            .context("No glyph for this character")?;
        let rect = face.outline_glyph(glyph_id, &mut outliner);
        dbg!(rect);
    }
    Ok(())
}
