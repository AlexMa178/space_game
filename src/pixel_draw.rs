use ggez::graphics::{ Canvas, Color, DrawParam, Image, Transform, ZIndex };

use generic_discrete_2d_rotations::Rot;

use crate::TILE_PIXELS;
use crate::scale::{ DPixelVector, PositionAndDimension, TPixelDimension, TTileDimension, TTileRect, TTileVector, ToPixel, ToRect, ToScreen, ToUV, WPixelVector };

pub enum AtlasSection {
    All, Rect { atlas_rect: TTileRect },
}
impl From<TTileRect> for AtlasSection {
    fn from(atlas_rect: TTileRect) -> Self {
        Self::Rect { atlas_rect }
    }
}

pub struct PixelDrawParams {
    pub camera: WPixelVector,
    pub atlas_section: AtlasSection,
    pub pos: WPixelVector,
    pub rot_pivot_tile: TTileVector,
    pub rotation: Rot<4>,
    pub z: ZIndex,
}
impl Default for PixelDrawParams {
    fn default() -> Self {
        Self {
            camera: WPixelVector::ZERO,
            atlas_section: AtlasSection::All,
            pos: WPixelVector::ZERO,
            rot_pivot_tile: TTileVector::ZERO,
            rotation: Rot::R4_0,
            z: 0,
        }
    }
}

pub fn pixel_draw(
    pixel_size: u8,
    canvas: &mut Canvas,
    image: &Image,
    params: PixelDrawParams,
) {

    let image_dim = TTileDimension::new(
        (image.width()  as i32 / TILE_PIXELS) as u8,
        (image.height() as i32 / TILE_PIXELS) as u8,
    );

    let atlas_section = match params.atlas_section {
        AtlasSection::All => TTileRect::from_pos_dim(TTileVector::ZERO, image_dim),
        AtlasSection::Rect { atlas_rect } => atlas_rect,
    };

    let half = DPixelVector::new(TILE_PIXELS / 2, TILE_PIXELS / 2);

    canvas.draw(image, DrawParam {
        src: atlas_section.to_pixel().to_uv(image_dim.to_pixel()).to_rect(),
        color: Color::WHITE,
        transform: Transform::Values {
            dest: (params.pos + half - params.camera).to_screen(pixel_size).into(),
            rotation: params.rotation.to_rad(),
            scale: TPixelDimension::ONE.to_screen(pixel_size).into(),
            offset: (params.rot_pivot_tile.to_pixel() + half).to_uv(atlas_section.dim().to_pixel()).into(),
        },
        z: params.z,
    });

}