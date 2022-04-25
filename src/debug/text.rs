use std::collections::HashMap;
use std::ops::Div;
use json::JsonValue;
use png::{BitDepth, ColorType};

use crate::prelude::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct TextSection<'a> {
    pub text: &'a str,
    pub color: TextColor,
    pub style: TextStyle,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct TextColor {
    pub r: u8,
    pub g: u8,
    pub b: u8
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TextStyle {
    Regular,
    Bold,
    Italic,
    BoldItalic,
}

pub struct TextGenerator {
    style: FontStyle,
    baseline_offset: f32,
}

impl TextGenerator {
    pub fn new(style: FontStyle) -> Self {
        let baseline_offset = style.ascender;

        Self {
            style,
            baseline_offset,
        }
    }

    pub fn generate(&self, text: &[TextSection]) -> (Box<[CharacterVertexData]>, [f32; 2]) {
        let mut vertex_data = Vec::with_capacity(text.len());
        let mut size = [0f32, 0f32];

        let mut head = Vec2f32::new(0f32, self.baseline_offset);
        for section in text.iter() {
            let style = self.get_style(section.style);

            if size[1] > head[1] + style.descender {
                size[1] = head[1] + style.descender;
            }

            for character in section.text.chars() {
                let (data, advance) = style.generate_character_vertex_data(character.into(), &section.color, &head);
                if let Some(data) = data {
                    vertex_data.push(data);
                }

                head[0] += advance;
            }
        }
        size[0] = head[0];

        (vertex_data.into_boxed_slice(), size)
    }

    fn get_style(&self, _: TextStyle) -> &FontStyle {
        &self.style
    }
}

pub struct FontStyle {
    atlas_index: u8,
    line_height: f32,
    ascender: f32,
    descender: f32,
    glyphs: HashMap<u32, FontGlyph>,
    replacement_glyph: FontGlyph,
}

impl FontStyle {
    pub fn from_json(data: &JsonValue, atlas_index: u8, texture_size: Vec2u32) -> Option<Self> {
        if let JsonValue::Object(object) = data {
            let line_height;
            let ascender;
            let descender;
            if let JsonValue::Object(atlas) = object.get("metrics")? {
                line_height = atlas.get("lineHeight")?.as_f32()?;
                ascender = atlas.get("ascender")?.as_f32()?;
                descender = atlas.get("descender")?.as_f32()?;
            } else {
                return None;
            }

            let mut glyphs;
            if let JsonValue::Array(glyph_data) = object.get("glyphs")? {
                glyphs = HashMap::with_capacity(glyph_data.len());

                for glyph in glyph_data {
                    let (code, glyph) = FontGlyph::from_json(glyph, texture_size)?;
                    glyphs.insert(code, glyph);
                }
            } else {
                return None;
            }

            let replacement_glyph = *glyphs.get(&0xFFFDu32)?;

            Some(Self {
                atlas_index,
                line_height,
                ascender,
                descender,
                glyphs,
                replacement_glyph,
            })
        } else {
            None
        }
    }

    pub fn generate_character_vertex_data(&self, code: u32, color: &TextColor, offset: &Vec2f32) -> (Option<CharacterVertexData>, f32) {
        let glyph = self.get_glyph_or_replacement(code);

        if glyph.generates() {
            (Some(CharacterVertexData {
                box_offset: glyph.box_offset + offset,
                box_size: glyph.box_size,
                atlas_offset: glyph.atlas_offset,
                atlas_size: glyph.atlas_size,
                color_r: color.r,
                color_g: color.g,
                color_b: color.b,
                atlas_index: self.atlas_index
            }), glyph.advance)
        } else {
            (None, glyph.advance)
        }
    }

    pub fn get_glyph_or_replacement(&self, code: u32) -> &FontGlyph {
        self.get_glyph(code).unwrap_or(self.get_replacement_glyph())
    }

    pub fn get_glyph(&self, code: u32) -> Option<&FontGlyph> {
        self.glyphs.get(&code)
    }

    pub fn get_replacement_glyph(&self) -> &FontGlyph {
        &self.replacement_glyph
    }
}

#[derive(Copy, Clone, Debug)]
pub struct FontGlyph {
    advance: f32,
    box_offset: Vec2f32,
    box_size: Vec2f32,
    atlas_offset: Vec2f32,
    atlas_size: Vec2f32,
    generates: bool,
}

impl FontGlyph {
    pub fn from_json(data: &JsonValue, texture_size: Vec2u32) -> Option<(u32, Self)> {
        let texture_size: Vec2f32 = texture_size.cast();

        if let JsonValue::Object(object) = data {
            let code = object.get("unicode")?.as_u32()?;
            let advance = object.get("advance")?.as_f32()?;

            if object.get("planeBounds").is_none() {
                return Some((
                    code,
                    Self {
                        advance,
                        box_offset: Vec2f32::new(0f32, 0f32),
                        box_size: Vec2f32::new(0f32, 0f32),
                        atlas_offset: Vec2f32::new(0f32, 0f32),
                        atlas_size: Vec2f32::new(0f32, 0f32),
                        generates: false,
                    }
                ));
            }

            let (box_offset, box_size) = Self::parse_bounds(object.get("planeBounds")?)?;
            let (atlas_offset, atlas_size) = Self::parse_bounds(object.get("atlasBounds")?)?;
            let mut atlas_offset = atlas_offset.component_div(&texture_size);
            let mut atlas_size = atlas_size.component_div(&texture_size);

            atlas_offset[1] = 1.0 - atlas_offset[1];
            atlas_size[1] *= -1.0;

            Some((
                code,
                Self {
                    advance,
                    box_offset,
                    box_size,
                    atlas_offset,
                    atlas_size,
                    generates: true
                }
            ))
        } else {
            None
        }
    }

    fn parse_bounds(bounds: &JsonValue) -> Option<(Vec2f32, Vec2f32)> {
        if let JsonValue::Object(bounds) = bounds {
            let left = bounds.get("left")?.as_f32()?;
            let bottom = bounds.get("bottom")?.as_f32()?;
            let right = bounds.get("right")?.as_f32()?;
            let top = bounds.get("top")?.as_f32()?;

            let offset = Vec2f32::new(left, bottom);
            let size = Vec2f32::new(right - left, top - bottom);

            Some((offset, size))
        } else {
            None
        }
    }

    pub fn generates(&self) -> bool {
        self.generates
    }

    pub fn get_advance(&self) -> f32 {
        self.advance
    }
}

/// The vertex data of a single character
#[repr(C)]
#[derive(Debug)]
pub struct CharacterVertexData {
    pub box_offset: Vec2f32,
    pub box_size: Vec2f32,
    pub atlas_offset: Vec2f32,
    pub atlas_size: Vec2f32,
    pub color_r: u8,
    pub color_g: u8,
    pub color_b: u8,
    pub atlas_index: u8,
}

pub fn ldfnt() {
    FontData::load();
}

pub struct FontData {
    pub regular_style: FontStyle,
    pub regular_image: (Box<[u8]>, (u32, u32)),
}

impl FontData {
    pub fn load() -> Self {
        let (regular_style, regular_img, regular_size) =
            Self::load_font(REGULAR_JSON, REGULAR_PNG, 0)
                .expect("Failed to load debug font regular");

        Self {
            regular_style,
            regular_image: (regular_img, regular_size),
        }
    }

    fn load_font(json: &str, png: &[u8], atlas_index: u8) -> Option<(FontStyle, Box<[u8]>, (u32, u32))> {
        let decoder = png::Decoder::new(png);
        let mut reader = decoder.read_info().map_err(
            |err| log::error!("Failed to read png info {:?}", err)).ok()?;

        if reader.info().is_animated() {
            log::error!("Font png has animation data. This is not supported");
            return None;
        }

        let (color_type, bit_depth) = reader.output_color_type();
        if color_type != ColorType::Rgb {
            log::error!("Font png has {:?} color type but only RGB is supported", color_type);
            return None;
        }
        if bit_depth != BitDepth::Eight {
            log::error!("Font png has {:?} bit depth but only 8 is supported", bit_depth);
            return None;
        }

        let mut buff = vec![0u8; reader.output_buffer_size()];
        let output = reader.next_frame(&mut buff).map_err(
            |err| log::error!("Failed to read png frame {:?}", err)).ok()?;
        let texture_size = (output.width, output.height);

        let json = json::parse(json).map_err(
            |err| log::error!("Failed to parse font json {:?}", err)).ok()?;
        let style = FontStyle::from_json(&json, atlas_index, Vec2u32::new(output.width, output.height)).or_else(
            || { log::error!("Failed to load style from json"); None })?;

        Some((style, buff.into_boxed_slice(), texture_size))
    }
}

const REGULAR_JSON: &'static str = include_str!(concat!(env!("B4D_RESOURCE_DIR"), "debug/regular.json"));
const REGULAR_PNG: &'static [u8] = include_bytes!(concat!(env!("B4D_RESOURCE_DIR"), "debug/regular.png"));