use crate::{ SCREEN_TILES, TILE_PIXELS };
use crate::scale::{ DPixelVector, ToPixel, WPixelVector, WTileDimension };
use crate::level::Level;

pub const MAX_OFFSET: i32 = 4 * TILE_PIXELS;

pub fn find_camera_pos(player_pos: WPixelVector, player_vel: DPixelVector, level_dim: WTileDimension) -> WPixelVector {
    let screen_pixels = SCREEN_TILES.to_pixel();
    let center = player_pos - screen_pixels / 2;
    let offset = player_vel.clamp(DPixelVector::splat(-MAX_OFFSET), DPixelVector::splat(MAX_OFFSET));
    let clamped = (center + offset).clamp(WPixelVector::ZERO, level_dim.to_pixel() - screen_pixels);
    clamped
}

pub fn initial(level: &Level) -> WPixelVector {
    find_camera_pos(level.spawn_player_pos.to_pixel(), DPixelVector::ZERO, level.collision.dim())
}