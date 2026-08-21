use std::collections::HashSet;

use ggez::graphics::Image;

use glamour::{ Point2, Rect, Size2 };

use serde::Deserialize;

use generic_discrete_2d_rotations::Ray;

use ggez_pixel_canvas::{ PixelCanvas, PixelDrawParams };

use crate::{ TILE_PIXELS, Pixel, Tile };

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CollisionType {
    Empty, Wall, Bouncy { normal_dir: Ray<4> }
}
impl CollisionType {

    fn from_str(s: &str) -> Self {
        match s {
            "1" => Self::Empty,
            "2" => Self::Wall,
            "3" => Self::Bouncy { normal_dir: Ray::IY_UP_4    },
            "4" => Self::Bouncy { normal_dir: Ray::IY_RIGHT_4 },
            "5" => Self::Bouncy { normal_dir: Ray::IY_DOWN_4  },
            "6" => Self::Bouncy { normal_dir: Ray::IY_LEFT_4  },
            a => panic!("invalid collision data, found \"{a}\""),
        }
    }

}

#[derive(Debug)]
pub struct LevelCollision {
    grid: Vec<CollisionType>,
    dim: Size2<Tile>,
}
impl LevelCollision {

    pub fn dim(&self) -> Size2<Tile> {
        self.dim
    }

    pub fn at(&self, pos: Point2<Tile>) -> CollisionType {
        let Point2 { x: c, y: r } = pos;
        self.grid[ r as usize * self.dim.width as usize + c as usize ]
    }

    pub fn touching(&self, obj_rect: Rect<Pixel>) -> HashSet<CollisionType> {

        let obj_rect_tile = Rect::from_min_max(
            (obj_rect.min().as_::<f32>() / TILE_PIXELS as f32).floor().as_::<Tile>(),
            (obj_rect.max().as_::<f32>() / TILE_PIXELS as f32).ceil() .as_::<Tile>(),
        );
        let positions = obj_rect_tile.x_range().flat_map(|x| obj_rect_tile.y_range().map(move |y| Point2::<Tile>::new(x, y))).collect::<Vec<_>>();

        positions.into_iter().map(|pos| self.at(pos)).collect()

    }

    pub fn from_csv_text(s: &str) -> Self {
        let grid_2d = s
            .lines()
            .map(|row| row
                .split(',')
                .filter(|cell| !cell.is_empty())
                .map(CollisionType::from_str)
                .collect::<Vec<_>>()
            )
            .collect::<Vec<_>>();
        let dim = Size2::new(grid_2d[0].len() as u8, grid_2d.len() as u8);
        let grid = grid_2d.into_iter().flatten().collect();
        LevelCollision { grid, dim }
    }

}

#[derive(Debug)]
pub struct Level {
    pub tiles: Image,
    pub collision: LevelCollision,
    pub spawn_player_pos: Point2<Tile>,
    pub spawn_portal_pos: Point2<Tile>,
    pub spawn_black_holes_pos: Vec<Point2<Tile>>,
}
impl Level {

    pub fn from_file_parts(tiles: Image, collision: LevelCollision, data: LevelData) -> Self {
        Self {
            tiles,
            collision,
            spawn_player_pos: Point2::from_tuple(data.player_init),
            spawn_portal_pos: Point2::from_tuple(data.portal),
            spawn_black_holes_pos: data.black_holes.into_iter().map(Point2::from_tuple).collect(),
        }
    }

    pub fn draw(&self, canvas: &mut PixelCanvas, camera: Point2<Pixel>) {
        canvas.draw(&self.tiles, PixelDrawParams::default().dest(-camera).z(1));
    }

}

#[derive(Deserialize)]
pub struct LevelData {
    pub player_init: (u8, u8),
    pub portal: (u8, u8),
    pub black_holes: Vec<(u8, u8)>,
}