use glamour::{ Point2, Size2, Vector2 };

use crate::{ SCREEN_TILES, TILE_PIXELS };
use crate::scale::{ Pixel, Tile, ToPixel };
use crate::level::Level;

pub const MAX_OFFSET: i32 = 4 * TILE_PIXELS;

pub fn find_camera_pos(player_pos: Point2<Pixel>, player_vel: Vector2<Pixel>, level_dim: Size2<Tile>) -> Point2<Pixel> {
    let screen_pixels = SCREEN_TILES.to_pixel();
    let center = player_pos - (screen_pixels / 2).to_vector();
    let offset = player_vel.clamp(Vector2::splat(-MAX_OFFSET), Vector2::splat(MAX_OFFSET));
    (center + offset).clamp(Point2::ZERO, (level_dim.to_pixel() - screen_pixels).to_vector().to_point())
}

pub fn initial(level: &Level) -> Point2<Pixel> {
    find_camera_pos(level.spawn_player_pos.to_pixel(), Vector2::ZERO, level.collision.dim())
}