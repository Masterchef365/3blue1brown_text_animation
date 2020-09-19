use anyhow::{Context, Result};
use harfbuzz_rs::{self as hb, UnicodeBuffer};
use owned_ttf_parser::{self as ttf, GlyphId, OutlineBuilder};
mod lines;
use lines::{Lines as LinesRender, Vertex};
use lyon::path::{Path, Builder as PathBuilder};
use lyon::math::{point, Point};
use lyon::lyon_tessellation::{BasicVertexConstructor, VertexBuffers};

struct PathTranslator {
    path: PathBuilder,
}

impl PathTranslator {
    pub fn new() -> Self {
        Self {
            path: PathBuilder::new()
        }
    }
    
    pub fn finish(self) -> Path {
        self.path.build()
    }
}

impl OutlineBuilder for PathTranslator {
    fn move_to(&mut self, x: f32, y: f32) {
        self.path.move_to(point(x, y));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.path.line_to(point(x, y));
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.path.quadratic_bezier_to(point(x1, y1), point(x, y));
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.path.cubic_bezier_to(point(x1, y1), point(x2, y2), point(x, y));
    }

    fn close(&mut self) {
        self.path.close()
    }
}

struct WithColor([f32; 3]);

impl BasicVertexConstructor<Vertex> for WithColor {
    fn new_vertex(&mut self, position: Point) -> Vertex {
        Vertex {
            pos: [position.x, position.y],
            color: self.0,
        }
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

    //let mut vertex_buffers: VertexBuffers<Vertex, u16> = VertexBuffers::new();

    //let downscale = 5000.0;
    //let mut vertices = Vec::new();
    //let mut x_position = -0.8;
    for (_position, info) in positions.iter().zip(infos) {
        let mut outliner = PathTranslator::new();
        let _rect = ttf_face.outline_glyph(GlyphId(info.codepoint as u16), &mut outliner);
        dbg!(outliner.finish());
        //x_position += position.x_advance as f32 / downscale;
    }

    //wgpu_launchpad::launch::<LinesRender>(vertices);

    Ok(())
}
