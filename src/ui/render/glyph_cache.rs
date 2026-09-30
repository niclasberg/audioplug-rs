use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
    range::Range,
};

use parley::FontData;
use skrifa::{FontRef, MetadataProvider, instance::NormalizedCoord, outline::DrawSettings};

use crate::{
    core::{FxHashMap, PathBuilder, PathSegment, Point, Rect},
    ui::render::scene::GpuPathSegment,
};

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
struct GlyphKey {
    font_index: u32,
    glyph_id: u32,
    variation_hash: u64,
}

impl GlyphKey {
    pub fn new(font_index: u32, glyph_id: u32, normalized_coords: &[i16]) -> Self {
        let mut hasher = DefaultHasher::new();
        normalized_coords.hash(&mut hasher);
        let variation_hash = hasher.finish();

        Self {
            font_index,
            glyph_id,
            variation_hash,
        }
    }
}

pub struct CachedGlyph<'a> {
    pub id: usize,
    pub outline: &'a [GpuPathSegment],
    pub segment_bounds: &'a [Rect],
    pub glyph_bounds: Rect,
}

struct CachedGlyphData {
    id: usize,
    segment_range: Range<usize>,
    bounds: Rect,
}

pub struct GlyphCache {
    next_glyph_id: usize,
    segment_cache: Vec<PathSegment>,
    segments: Vec<GpuPathSegment>,
    segment_bounds: Vec<Rect>,
    glyphs: FxHashMap<GlyphKey, CachedGlyphData>,
}

impl GlyphCache {
    pub fn new() -> Self {
        Self {
            next_glyph_id: 0,
            segment_cache: Vec::new(),
            segments: Vec::new(),
            segment_bounds: Vec::new(),
            glyphs: HashMap::default(),
        }
    }

    pub fn get<'s>(
        &'s mut self,
        font: &FontData,
        glyph_id: u32,
        normalized_coords: &[i16],
    ) -> Option<CachedGlyph<'s>> {
        let key = GlyphKey::new(font.index, glyph_id, normalized_coords);
        if let Some(cached_glyph) = self.glyphs.get(&key) {
            return Some(CachedGlyph {
                id: cached_glyph.id,
                outline: &self.segments[cached_glyph.segment_range],
                segment_bounds: &self.segment_bounds[cached_glyph.segment_range],
                glyph_bounds: cached_glyph.bounds,
            });
        }

        let normalized_coords = bytemuck::cast_slice::<_, NormalizedCoord>(
            normalized_coords, // glyph_run.run().normalized_coords(),
        );

        let font_collection_ref = font.data.as_ref();
        let font_ref = FontRef::from_index(font_collection_ref, font.index).unwrap();
        let outline_glyphs = font_ref.outline_glyphs();
        let scale_factor = font_ref
            .metrics(skrifa::instance::Size::unscaled(), normalized_coords)
            .units_per_em as f32;

        if let Some(outline) = outline_glyphs.get(glyph_id.into()) {
            self.segment_cache.clear();
            let path_offset = self.segments.len();
            let mut pen = GlyphPen {
                builder: PathBuilder::new(&mut self.segment_cache),
                scale: 1.0 / scale_factor,
            };
            outline
                .draw(
                    DrawSettings::unhinted(skrifa::instance::Size::unscaled(), normalized_coords),
                    &mut pen,
                )
                .unwrap();
            self.segments
                .extend(self.segment_cache.iter().map(|p| match p {
                    PathSegment::Line(l) => GpuPathSegment::line(l.p0, l.p1),
                    PathSegment::Quad(q) => GpuPathSegment::quad(q.p0, q.p1, q.p2),
                    PathSegment::Cubic(c) => GpuPathSegment::cubic(c.p0, c.p1, c.p2, c.p3),
                }));
            let segment_range = Range {
                start: path_offset,
                end: self.segments.len(),
            };
            self.segment_bounds
                .extend(self.segment_cache.iter().map(PathSegment::bounds));
            let bounds = self.segment_bounds[segment_range]
                .iter()
                .fold(Rect::EMPTY, |acc, cur| acc.union(cur));
            let id = self.next_glyph_id;
            self.next_glyph_id += 1;

            self.glyphs.insert(
                key,
                CachedGlyphData {
                    id,
                    segment_range,
                    bounds,
                },
            );

            Some(CachedGlyph {
                id,
                outline: &self.segments[segment_range],
                segment_bounds: &self.segment_bounds[segment_range],
                glyph_bounds: bounds,
            })
        } else {
            None
        }
    }
}

struct GlyphPen<'a> {
    builder: PathBuilder<'a>,
    scale: f32,
}

impl GlyphPen<'_> {
    fn point(&self, x: f32, y: f32) -> Point {
        Point { x: x, y: -y }.scale(self.scale)
    }
}

impl<'a> skrifa::outline::OutlinePen for GlyphPen<'a> {
    fn move_to(&mut self, x: f32, y: f32) {
        self.builder.move_to(self.point(x, y));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.builder.line_to(self.point(x, y));
    }

    fn quad_to(&mut self, cx0: f32, cy0: f32, x: f32, y: f32) {
        self.builder.quad_to(self.point(cx0, cy0), self.point(x, y));
    }

    fn curve_to(&mut self, cx0: f32, cy0: f32, cx1: f32, cy1: f32, x: f32, y: f32) {
        self.builder
            .cubic_to(self.point(cx0, cy0), self.point(cx1, cy1), self.point(x, y));
    }

    fn close(&mut self) {
        self.builder.close_path();
    }
}
