use anyhow::{Context, Result};
use harfbuzz_rs::{self as hb, UnicodeBuffer};
use owned_ttf_parser::{self as ttf, GlyphId, OutlineBuilder};
mod lines;
use lines::{Lines as LinesRender, Vertex};

pub type Point = [f32; 2];
pub type Color = [f32; 3];
pub type Line = (Point, Point, Color);

struct Outliner {
    color: Color,
    lines: Vec<Line>,
    last: Option<Point>,
    first: Option<Point>,
}

impl Outliner {
    pub fn new(color: [f32; 3]) -> Self {
        Self {
            color,
            lines: Vec::new(),
            first: None,
            last: None,
        }
    }

    fn last(&self) -> Point {
        self.last.expect("No initial MoveTo!")
    }

    pub fn lines(self) -> Vec<Line> {
        self.lines
    }
}

impl OutlineBuilder for Outliner {
    fn move_to(&mut self, x: f32, y: f32) {
        self.last = Some([x, y]);
        self.first = Some([x, y]);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let pt = [x, y];
        self.lines.push((self.last(), pt, self.color));
        self.last = Some(pt);
    }

    fn quad_to(&mut self, _x1: f32, _y1: f32, x: f32, y: f32) {
        let pt = [x, y];
        self.lines.push((self.last(), pt, self.color));
        self.last = Some(pt);
    }

    fn curve_to(&mut self, _x1: f32, _y1: f32, _x2: f32, _y2: f32, x: f32, y: f32) {
        let pt = [x, y];
        self.lines.push((self.last(), pt, self.color));
        self.last = Some(pt);
    }

    fn close(&mut self) {
        /*self.lines.push((
            self.last(),
            self.first.expect("No first point!"),
            self.color,
        ));*/
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

        let downscale = 5000.0;
    let mut vertices = Vec::new();
    let mut x_position = -0.8;
    for (position, info) in positions.iter().zip(infos) {
        let mut outliner = Outliner::new([1.0, 1.0, 1.0]);
        let _rect = ttf_face.outline_glyph(GlyphId(info.codepoint as u16), &mut outliner);

        for (a, b, color) in outliner.lines.iter().copied() {
            let a = [a[0] / downscale + x_position, a[1] / downscale]; 
            let b = [b[0] / downscale + x_position, b[1] / downscale]; 
            vertices.push(Vertex {
                pos: a,
                color,
            });
            vertices.push(Vertex {
                pos: b,
                color,
            });
        }

        x_position += position.x_advance as f32 / downscale;
    }

    wgpu_launchpad::launch::<LinesRender>(vertices);

    Ok(())
}
