use std::collections::HashSet;

use ggez::graphics::Image;

use serde::Deserialize;

use generic_discrete_2d_rotations::Ray;

use ggez_pixel_canvas::{ PDPBuilder, PixelCanvas };

use crate::TILE_PIXELS;
use crate::scale::{ PositionAndDimension, WPixelRect, WPixelVector, WTileDimension, WTileVector };

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
    dim: WTileDimension,
}
impl LevelCollision {

    pub fn dim(&self) -> WTileDimension {
        self.dim
    }

    pub fn at(&self, pos: WTileVector) -> CollisionType {
        let [ c, r ] = pos.as_usizevec2().into();
        self.grid[ r * self.dim.x as usize + c ]
    }

    pub fn touching(&self, obj_rect: WPixelRect) -> HashSet<CollisionType> {

        let min_pixel = obj_rect.pos();
        let max_pixel = obj_rect.pos() + obj_rect.dim();
        let min = (min_pixel / TILE_PIXELS).as_u8vec2();
        let max = (max_pixel / TILE_PIXELS).as_u8vec2() + (max_pixel % TILE_PIXELS).map(|i| (i != 0) as i32).as_u8vec2();
        let positions = ((min.x)..(max.x)).flat_map(|x| ((min.y)..(max.y)).map(move |y| WTileVector::new(x, y))).collect::<Vec<_>>();

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
        let dim = WTileDimension::new(grid_2d[0].len() as u8, grid_2d.len() as u8);
        let grid = grid_2d.into_iter().flatten().collect();
        LevelCollision { grid, dim }
    }

}

#[derive(Debug)]
pub struct Level {
    pub tiles: Image,
    pub collision: LevelCollision,
    pub spawn_player_pos: WTileVector,
    pub spawn_portal_pos: WTileVector,
    pub spawn_black_holes_pos: Vec<WTileVector>,
}
impl Level {

    pub fn from_file_parts(tiles: Image, collision: LevelCollision, data: LevelData) -> Self {
        Self {
            tiles,
            collision,
            spawn_player_pos: WTileVector::from(data.player_init),
            spawn_portal_pos: WTileVector::from(data.portal),
            spawn_black_holes_pos: data.black_holes.into_iter().map(WTileVector::from).collect(),
        }
    }

    pub fn draw(&self, canvas: &mut PixelCanvas, camera: WPixelVector) {
        canvas.draw(&self.tiles, PDPBuilder::<i32>::new().dest(-camera).z(1).build());
    }

}

#[derive(Deserialize)]
pub struct LevelData {
    pub player_init: (u8, u8),
    pub portal: (u8, u8),
    pub black_holes: Vec<(u8, u8)>,
}