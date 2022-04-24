use std::collections::HashMap;
use json::JsonValue;
use png::{BitDepth, ColorType};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct TextSection<'a> {
    text: &'a str,
    color: TextColor,
    style: TextStyle,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct TextColor {
    r: u8,
    g: u8,
    b: u8
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum TextStyle {
    Regular,
    Bold,
    Italic,
    BoldItalic,
}

struct TextGenerator {
    baseline_offset: f32,
}

impl TextGenerator {
    fn generate(&self, text: &[TextSection]) -> (Box<[CharacterVertexData]>, [f32; 2]) {
        let mut vertex_data = Vec::with_capacity(text.len());
        let mut size = [0f32, 0f32];

        let mut head = [0f32, self.baseline_offset];
        for section in text.iter() {
            let style = self.get_style(section.style);

            if size[1] > head[1] + style.descender {
                size[1] = head[1] + style.descender;
            }

            for character in section.text.chars() {
                let glyph = style.get_glyph_or_replacement(character.into());

                if glyph.generates() {
                    vertex_data.push(CharacterVertexData{
                        box_offset: glyph.generate_box_offset(&head),
                        box_size: glyph.box_size,
                        atlas_offset: glyph.atlas_offset,
                        atlas_size: glyph.atlas_size,
                        color_r: section.color.r,
                        color_g: section.color.g,
                        color_b: section.color.b,
                        atlas_index: style.atlas_index
                    })
                }

                head[0] += glyph.get_advance();
            }
        }
        size[0] = head[0];

        (vertex_data.into_boxed_slice(), size)
    }

    fn get_style(&self, style: TextStyle) -> &FontStyle {
        todo!()
    }
}

struct FontStyle {
    atlas_index: u8,
    line_height: f32,
    ascender: f32,
    descender: f32,
    glyphs: HashMap<u32, FontGlyph>,
    replacement_glyph: FontGlyph,
}

impl FontStyle {
    pub fn from_json(data: &JsonValue, atlas_index: u8) -> Option<Self> {
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
                    let (code, glyph) = FontGlyph::from_json(glyph)?;
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
struct FontGlyph {
    advance: f32,
    box_offset: [f32; 2],
    box_size: [f32; 2],
    atlas_offset: [f32; 2],
    atlas_size: [f32; 2],
}

impl FontGlyph {
    pub fn from_json(data: &JsonValue) -> Option<(u32, Self)> {
        if let JsonValue::Object(object) = data {
            let code = object.get("unicode")?.as_u32()?;
            let advance = object.get("advance")?.as_f32()?;

            if object.get("planeBounds").is_none() {
                return Some((
                    code,
                    Self {
                        advance,
                        box_offset: [0f32, 0f32],
                        box_size: [0f32, 0f32],
                        atlas_offset: [0f32, 0f32],
                        atlas_size: [0f32, 0f32],
                    }
                ));
            }

            let (box_offset, box_size) = Self::parse_bounds(object.get("planeBounds")?)?;
            let (atlas_offset, atlas_size) = Self::parse_bounds(object.get("atlasBounds")?)?;

            Some((
                code,
                Self {
                    advance,
                    box_offset,
                    box_size,
                    atlas_offset,
                    atlas_size
                }
            ))
        } else {
            None
        }
    }

    fn parse_bounds(bounds: &JsonValue) -> Option<([f32; 2], [f32; 2])> {
        if let JsonValue::Object(bounds) = bounds {
            let left = bounds.get("left")?.as_f32()?;
            let bottom = bounds.get("bottom")?.as_f32()?;
            let right = bounds.get("right")?.as_f32()?;
            let top = bounds.get("top")?.as_f32()?;

            let offset = [left, bottom];
            let size = [right - left, top - bottom];

            Some((offset, size))
        } else {
            None
        }
    }

    pub fn generates(&self) -> bool {
        self.box_size[0] == 0f32
    }

    pub fn get_advance(&self) -> f32 {
        self.advance
    }

    pub fn generate_box_offset(&self, offset: &[f32; 2]) -> [f32; 2] {
        [self.box_offset[0] + offset[0], self.box_offset[1] + offset[1]]
    }
}

/// The vertex data of a single character
#[repr(C)]
struct CharacterVertexData {
    box_offset: [f32; 2],
    box_size: [f32; 2],
    atlas_offset: [f32; 2],
    atlas_size: [f32; 2],
    color_r: u8,
    color_g: u8,
    color_b: u8,
    atlas_index: u8,
}

pub fn ldfnt() {
    FontData::load();
}

struct FontData {
    regular_style: FontStyle,
    regular_image: (Box<[u8]>, (u32, u32)),
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
        let json = json::parse(json).map_err(
            |err| log::error!("Failed to parse font json {:?}", err)).ok()?;
        let style = FontStyle::from_json(&json, atlas_index).or_else(
            || { log::error!("Failed to load style from json"); None })?;

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

        Some((style, buff.into_boxed_slice(), (output.width, output.height)))
    }
}

fn load_font() -> (Box<[u8]>, FontStyle) {
    let json = json::parse(REGULAR_JSON).unwrap();
    let style = FontStyle::from_json(&json, 0).unwrap();

    let decoder = png::Decoder::new(REGULAR_PNG);
    let mut reader = decoder.read_info().unwrap();

    let mut buff = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buff).unwrap();

    buff.resize(info.buffer_size(), 0);

    (buff.into_boxed_slice(), style)
}

const REGULAR_JSON: &'static str = include_str!(concat!(env!("B4D_RESOURCE_DIR"), "debug/regular.json"));
const REGULAR_PNG: &'static [u8] = include_bytes!(concat!(env!("B4D_RESOURCE_DIR"), "debug/regular.png"));