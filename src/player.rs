use std::collections::HashSet;

use ggez::winit::keyboard::KeyCode;

use generic_discrete_2d_rotations::{ A4, A8, Angle, Ray, RotDir, RotFrom };

use ggez_pixel_canvas::{ PDPTileBuilder, PixelCanvas };

use crate::TILE_PIXELS;
use crate::animated::BlackHole;
use crate::level::{CollisionType, LevelCollision};
use crate::scale::{ DPixelVector, PositionAndDimension, TTileDimension, TTileRect, TTileVector, ToPixel, WPixelRect, WPixelVector, WTileDimension, WTileVector };
use crate::assets::{ PLAYER_IMAGE };
use crate::sounds::AllSounds;

pub const PLAYER_ACCELERATION: i32 = 2;
pub const PLAYER_DECELERATION: i32 = 1;

pub struct Player {
    pos: WPixelVector,
    vel: DPixelVector,
    dir: Ray<8>,
    fire: bool,
}
impl Player {

    pub fn new(initial_pos: WTileVector) -> Self {

        Self {
            pos: initial_pos.to_pixel(),
            vel: DPixelVector::ZERO,
            dir: Ray::IY_UP_8,
            fire: false,
        }

    }

    pub fn pos(&self) -> WPixelVector {
        self.pos
    }

    pub fn rect(&self) -> WPixelRect {
        WPixelRect::from_pos_dim(self.pos, WTileDimension::ONE.to_pixel())
    }

    pub fn vel(&self) -> DPixelVector {
        self.vel
    }

    pub fn update(&mut self, sounds: &mut AllSounds, subframe_counter: u8, num_subframes: u8, keys: HashSet<KeyCode>, level_collision: &LevelCollision, black_holes: &Vec<BlackHole>) {

        let collisions = level_collision.touching(self.rect());
        let maybe_normal_dir = collisions.iter().find_map(|c| if let CollisionType::Bouncy { normal_dir } = c { Some(*normal_dir) } else { None });
        if let Some(normal_dir) = maybe_normal_dir {
            sounds.play_bounce();
            self.dir = normal_dir.embed_as::<8>();
            match normal_dir.rot(RotFrom::NegY, RotDir::CounterClockwise).matchable_4() {
                A4::D0   => self.vel.y = -self.vel.y.abs(),
                A4::D90  => self.vel.x =  self.vel.x.abs(),
                A4::D180 => self.vel.y =  self.vel.y.abs(),
                A4::D270 => self.vel.x = -self.vel.x.abs(),
            }
        }

        let u = keys.contains(&KeyCode::ArrowUp   ) || keys.contains(&KeyCode::KeyW);
        let d = keys.contains(&KeyCode::ArrowDown ) || keys.contains(&KeyCode::KeyS);
        let l = keys.contains(&KeyCode::ArrowLeft ) || keys.contains(&KeyCode::KeyA);
        let r = keys.contains(&KeyCode::ArrowRight) || keys.contains(&KeyCode::KeyD);

        let v = d as i32 - u as i32;
        let h = r as i32 - l as i32;

        let maybe_dir = Ray::<8>::from_signs(h.cmp(&0), v.cmp(&0));
        self.fire = maybe_dir.is_some();
        if let Some(dir) = maybe_dir {
            self.dir = dir;
        };

        if subframe_counter == 0 {

            if h == 0 {
                self.vel.x = if self.vel.x.abs() < PLAYER_DECELERATION { 0 } else { self.vel.x - self.vel.x.signum() * PLAYER_DECELERATION };
            } else {
                self.vel.x += h * PLAYER_ACCELERATION;
            }

            if v == 0 {
                self.vel.y = if self.vel.y.abs() < PLAYER_DECELERATION { 0 } else { self.vel.y - self.vel.y.signum() * PLAYER_DECELERATION };
            } else {
                self.vel.y += v * PLAYER_ACCELERATION;
            }

            let total_bh_influence = black_holes.iter().fold(DPixelVector::ZERO, |acc, bh| acc + bh.influence(self.rect()));
            self.vel += total_bh_influence;
            if total_bh_influence.length_squared() != 0 {
                let volume = (total_bh_influence.as_vec2().length() / 10.).clamp(0., 1.);
                sounds.play_black_hole_pull(volume);
            }

        }

        let per_subframe_x = self.vel.x / num_subframes as i32;
        let per_subframe_y = self.vel.y / num_subframes as i32;
        
        self.pos += DPixelVector::new(per_subframe_x, per_subframe_y);
        
        let rem_x = self.vel.x % num_subframes as i32;
        let mut arr_x = vec![false; num_subframes as usize];
        if rem_x != 0 {
            let diff_x = num_subframes as f32 / rem_x.abs() as f32;
            for s in 0..num_subframes {
                let i = (diff_x * s as f32).floor() as usize;
                if i < num_subframes as usize {
                    arr_x[i] = true;
                }
            }
        }
        assert_eq!(rem_x.abs() as usize, arr_x.iter().filter(|x| **x).count());
        if arr_x[subframe_counter as usize] {
            self.pos.x += rem_x.signum();
        }

        let rem_y = self.vel.y % num_subframes as i32;
        let mut arr_y = vec![false; num_subframes as usize];
        if rem_y != 0 {
            let diff_y = num_subframes as f32 / rem_y.abs() as f32;
            for s in 0..num_subframes {
                let i = (diff_y * s as f32).floor() as usize;
                if i < num_subframes as usize {
                    arr_y[i] = true;
                }
            }
        }
        assert_eq!(rem_y.abs() as usize, arr_y.iter().filter(|x| **x).count());
        if arr_y[subframe_counter as usize] {
            self.pos.y += rem_y.signum();
        }

    }

    pub fn draw(&self, canvas: &mut PixelCanvas, camera: WPixelVector) {

        let atlas_pos = match self.dir.rot(RotFrom::NegY, RotDir::CounterClockwise).matchable_8() {
            A8::D0   => TTileVector::new(1, 0),
            A8::D45  => TTileVector::new(2, 0),
            A8::D90  => TTileVector::new(2, 1),
            A8::D135 => TTileVector::new(2, 2),
            A8::D180 => TTileVector::new(1, 2),
            A8::D225 => TTileVector::new(0, 2),
            A8::D270 => TTileVector::new(0, 1),
            A8::D315 => TTileVector::new(0, 0),
        };

        canvas.draw(PLAYER_IMAGE.get(), PDPTileBuilder::<i32, u8>::new(TILE_PIXELS)
            .pixel_dest(self.pos - camera)
            .tile_atlas_rect(TTileRect::from_pos_dim(atlas_pos, TTileDimension::ONE))
            .z(2)
        .build());

        if self.fire {

            let (atlas_rect, rot_pivot_tile, rotation) = if let Some(rot_4) = self.dir.split_as::<4>() {
                (TTileRect::new(3, 0, 1, 2), TTileVector::new(0, 0), rot_4.rot(RotFrom::NegY, RotDir::CounterClockwise))
            } else {
                (TTileRect::new(4, 0, 2, 2), TTileVector::new(1, 0), (self.dir.rot(RotFrom::NegY, RotDir::CounterClockwise) - Angle::A8_45).split_as::<4>().unwrap())
            };

            canvas.draw(PLAYER_IMAGE.get(), PDPTileBuilder::<i32, u8>::new(TILE_PIXELS)
                .pixel_dest(self.pos - camera)
                .tile_atlas_rect(atlas_rect)
                .angle(rotation)
                .tile_anchor(rot_pivot_tile)
                .tile_pivot(rot_pivot_tile, true, true)
                .z(2)
            .build());

        }

    }

}