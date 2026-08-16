use ggez::graphics::Rect;

use ggez::glam::{ IVec2, IVec4, U8Vec2, U8Vec4, Vec2, Vec4 };

use crate::TILE_PIXELS;

pub type WTileVector = U8Vec2;
pub type DTileVector = U8Vec2;
pub type TTileVector = U8Vec2;
pub type WTileDimension = U8Vec2;
pub type TTileDimension = U8Vec2;
pub type TTileRect = U8Vec4;

pub type WPixelVector = IVec2;
pub type DPixelVector = IVec2;
pub type TPixelDimension = IVec2;
pub type WPixelRect = IVec4;

pub trait ToPixel {
    type Output;
    fn to_pixel(self) -> Self::Output;
}
impl ToPixel for U8Vec2 {
    type Output = IVec2;
    fn to_pixel(self) -> Self::Output {
        self.as_ivec2() * TILE_PIXELS
    }
}
impl ToPixel for U8Vec4 {
    type Output = IVec4;
    fn to_pixel(self) -> Self::Output {
        self.as_ivec4() * TILE_PIXELS
    }
}

pub trait ToScreen {
    type Output;
    fn to_screen(self, pixel_size: u8) -> Self::Output;
}
impl ToScreen for IVec2 {
    type Output = Vec2;
    fn to_screen(self, pixel_size: u8) -> Self::Output {
        self.as_vec2() * pixel_size as f32
    }
}

pub trait ToUV {
    type Output;
    fn to_uv(self, t_size: TPixelDimension) -> Self::Output;
}
impl ToUV for IVec2 {
    type Output = Vec2;
    fn to_uv(self, t_size: TPixelDimension) -> Self::Output {
        self.as_vec2() / t_size.as_vec2()
    }
}
impl ToUV for IVec4 {
    type Output = Vec4;
    fn to_uv(self, t_size: TPixelDimension) -> Self::Output {
        Vec4::from_pos_dim(self.pos().to_uv(t_size), self.dim().to_uv(t_size))
    }
}

pub trait PositionAndDimension<T>: Into<[T; 4]> + From<[T; 4]> {
    type Vec2Type: Into<[T; 2]> + From<[T; 2]>;
    fn from_pos_dim(pos: Self::Vec2Type, dim: Self::Vec2Type) -> Self {
        let [ x, y ] = pos.into();
        let [ w, h ] = dim.into();
        [ x, y, w, h ].into()
    }
    fn pos(self) -> Self::Vec2Type {
        let [ x, y, _w, _h ] = self.into();
        [ x, y ].into()
    }
    fn dim(self) -> Self::Vec2Type {
        let [ _x, _y, w, h ] = self.into();
        [ w, h ].into()
    }
}
impl PositionAndDimension<u8> for U8Vec4 {
    type Vec2Type = U8Vec2;
}
impl PositionAndDimension<i32> for IVec4 {
    type Vec2Type = IVec2;
}
impl PositionAndDimension<f32> for Vec4 {
    type Vec2Type = Vec2;
}

pub trait ToRect {
    fn to_rect(self) -> Rect;
}
impl ToRect for Vec4 {
    fn to_rect(self) -> Rect {
        let [ x, y, w, h ] = self.into();
        Rect { x, y, w, h }
    }
}