#[derive(Debug)]
pub struct TextureCoordinates {
    pub u: f32,
    pub v: f32,
}

pub struct SpritePosition {
    pub x: u8,
    pub y: u8,
}
impl SpritePosition {
    pub const fn new(x: u8, y: u8) -> Self {
        SpritePosition { x, y }
    }
}

#[derive(Debug, Clone)]
pub struct SpriteSheetDimensions {
    rows: u8,
    columns: u8,
}
impl SpriteSheetDimensions {
    pub fn new(rows: u8, columns: u8) -> Self {
        Self { rows, columns }
    }
}

#[derive(Debug)]
pub struct SpriteSheet {
    texture: u32,
    pub sprites_per_row: u8,
    pub sprites_per_column: u8,
}
impl Default for SpriteSheet {
    fn default() -> Self {
        Self {
            texture: 0,
            sprites_per_row: 1,
            sprites_per_column: 1,
        }
    }
}
impl SpriteSheet {
    pub fn texture(&self) -> u32 {
        self.texture
    }
    pub fn new(texture: u32, dimensions: &SpriteSheetDimensions) -> Self {
        Self {
            texture,
            sprites_per_row: dimensions.rows,
            sprites_per_column: dimensions.columns,
        }
    }
    pub fn get_sprite_coordinates(&self, position: &SpritePosition) -> [TextureCoordinates; 4] {
        let width = 1.0 / self.sprites_per_row as f32;
        let height = 1.0 / self.sprites_per_column as f32;
        let x_offset = position.x as f32 * width;
        let y_offset = position.y as f32 * height;
        [
            TextureCoordinates {
                u: x_offset,
                v: y_offset,
            },
            TextureCoordinates {
                u: x_offset + width,
                v: y_offset,
            },
            TextureCoordinates {
                u: x_offset + width,
                v: y_offset + height,
            },
            TextureCoordinates {
                u: x_offset,
                v: y_offset + height,
            },
        ]
    }
}
