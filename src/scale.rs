use glamour::{ Box2, Point2, Rect, Size2, Unit, Vector2 };

use crate::TILE_PIXELS;

pub struct Tile;
impl Unit for Tile {
    type Scalar = u8;
}

pub struct Pixel;
impl Unit for Pixel {
    type Scalar = i32;
}

pub struct ScreenPos;
impl Unit for ScreenPos {
    type Scalar = f32;
}

pub trait ToPixel {
    type Output;
    fn to_pixel(self) -> Self::Output;
}
impl ToPixel for Point2<Tile> {
    type Output = Point2<Pixel>;
    fn to_pixel(self) -> Self::Output {
        self.as_::<Pixel>() * TILE_PIXELS
    }
}
impl ToPixel for Vector2<Tile> {
    type Output = Vector2<Pixel>;
    fn to_pixel(self) -> Self::Output {
        self.as_::<Pixel>() * TILE_PIXELS
    }
}
impl ToPixel for Size2<Tile> {
    type Output = Size2<Pixel>;
    fn to_pixel(self) -> Self::Output {
        self.as_::<Pixel>() * TILE_PIXELS
    }
}
impl ToPixel for Rect<Tile> {
    type Output = Rect<Pixel>;
    fn to_pixel(self) -> Self::Output {
        Box2::from_array(self.as_::<Pixel>().to_box2().to_array().map(|p| p * TILE_PIXELS)).to_rect()
    }
}

pub trait ToScreen {
    type Output;
    fn to_screen(self, pixel_size: u8) -> Self::Output;
}
impl ToScreen for Size2<Pixel> {
    type Output = Size2<ScreenPos>;
    fn to_screen(self, pixel_size: u8) -> Self::Output {
        self.as_::<ScreenPos>() * pixel_size as f32
    }
}