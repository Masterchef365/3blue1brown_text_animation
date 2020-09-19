use anyhow::{Context, Result};
use harfbuzz_rs::{self as hb, UnicodeBuffer};
use owned_ttf_parser::{self as ttf, GlyphId, OutlineBuilder};
mod render;
use lyon::lyon_tessellation::{
    BuffersBuilder, FillAttributes, FillOptions, FillTessellator, FillVertexConstructor,
    StrokeAttributes, StrokeOptions, StrokeTessellator, StrokeVertexConstructor, VertexBuffers,
};
use lyon::math::{point, Point};
use lyon::path::{Builder as PathBuilder, Path};
use render::{Args, Renderer, Vertex};

struct PathTranslator {
    path: PathBuilder,
}

impl PathTranslator {
    pub fn new() -> Self {
        Self {
            path: PathBuilder::new(),
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
        self.path
            .cubic_bezier_to(point(x1, y1), point(x2, y2), point(x, y));
    }

    fn close(&mut self) {
        self.path.close()
    }
}

struct FillVertexCtor {
    offset: Point,
    scaling: f32,
}

impl FillVertexConstructor<Vertex> for FillVertexCtor {
    fn new_vertex(&mut self, position: Point, _: FillAttributes) -> Vertex {
        let Point { x, y, .. } = position + self.offset.to_vector();
        Vertex {
            pos: [x * self.scaling, y * self.scaling],
            value: 0.0,
        }
    }
}

struct StrokeVertexCtor {
    offset: Point,
    scaling: f32,
    value: f32,
}

impl StrokeVertexConstructor<Vertex> for StrokeVertexCtor {
    fn new_vertex(&mut self, position: Point, _: StrokeAttributes) -> Vertex {
        let Point { x, y, .. } = position + self.offset.to_vector();
        self.value += 1.0;
        Vertex {
            pos: [x * self.scaling, y * self.scaling],
            value: self.value,
        }
    }
}

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let ttf_path = args.next().context("Requires TTF path")?;
    let scaling = args
        .next()
        .context("Requires text")
        .and_then(|arg| Ok(arg.parse::<f32>()?))?;
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

    let mut fill_buf: VertexBuffers<Vertex, u16> = VertexBuffers::new();
    let mut fill_tess = FillTessellator::new();
    let mut stroke_buf: VertexBuffers<Vertex, u16> = VertexBuffers::new();
    let mut stroke_tess = StrokeTessellator::new();

    let mut x_position = 0.0;
    let y_position = 5000.0;
    for (position, info) in positions.iter().zip(infos) {
        let mut outliner = PathTranslator::new();
        let _rect = ttf_face.outline_glyph(GlyphId(info.codepoint as u16), &mut outliner);
        let path = outliner.finish();

        let offset = point(x_position, y_position);

        // Fill
        let ctor = FillVertexCtor { offset, scaling };
        let mut builder = BuffersBuilder::new(&mut fill_buf, ctor);
        fill_tess
            .tessellate(&path, &FillOptions::tolerance(5.), &mut builder)
            .unwrap();

        // Stroke
        let ctor = StrokeVertexCtor {
            offset,
            scaling,
            value: 0.0,
        };
        let mut builder = BuffersBuilder::new(&mut stroke_buf, ctor);
        stroke_tess
            .tessellate(
                &path,
                &StrokeOptions::tolerance(5.).with_line_width(25.),
                &mut builder,
            )
            .unwrap();

        x_position += position.x_advance as f32;
    }

    let args = Args {
        fill_vertices: fill_buf.vertices,
        fill_indices: fill_buf.indices,
        stroke_vertices: stroke_buf.vertices,
        stroke_indices: stroke_buf.indices,
    };

    wgpu_launchpad::launch::<Renderer>(args);

    Ok(())
}
