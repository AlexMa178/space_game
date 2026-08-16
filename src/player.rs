use std::collections::HashSet;

use ggez::graphics::Canvas;

use ggez::winit::keyboard::KeyCode;

use generic_discrete_2d_rotations::{ R4, R8, Rot, RotDir, RotFrom };

use crate::animated::BlackHole;
use crate::level::{CollisionType, LevelCollision};
use crate::scale::{ DPixelVector, PositionAndDimension, TTileDimension, TTileRect, TTileVector, ToPixel, WPixelRect, WPixelVector, WTileDimension, WTileVector };
use crate::pixel_draw::{ self, PixelDrawParams };
use crate::assets::{ PLAYER_IMAGE };
use crate::sounds::AllSounds;

pub const PLAYER_ACCELERATION: i32 = 2;
pub const PLAYER_DECELERATION: i32 = 1;

pub struct Player {
    pos: WPixelVector,
    vel: DPixelVector,
    rot: Rot<8>,
    fire: bool,
}
impl Player {

    pub fn new(initial_pos: WTileVector) -> Self {

        Self {
            pos: initial_pos.to_pixel(),
            vel: DPixelVector::ZERO,
            rot: Rot::R8_0,
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
            self.rot = normal_dir.embed_as::<8>();
            match normal_dir.matchable_4() {
                R4::D0   => self.vel.y = -self.vel.y.abs(),
                R4::D90  => self.vel.x =  self.vel.x.abs(),
                R4::D180 => self.vel.y =  self.vel.y.abs(),
                R4::D270 => self.vel.x = -self.vel.x.abs(),
            }
        }

        let u = keys.contains(&KeyCode::ArrowUp   ) || keys.contains(&KeyCode::KeyW);
        let d = keys.contains(&KeyCode::ArrowDown ) || keys.contains(&KeyCode::KeyS);
        let l = keys.contains(&KeyCode::ArrowLeft ) || keys.contains(&KeyCode::KeyA);
        let r = keys.contains(&KeyCode::ArrowRight) || keys.contains(&KeyCode::KeyD);

        let v = d as i32 - u as i32;
        let h = r as i32 - l as i32;

        let maybe_rot = Rot::<8>::from_signs(RotFrom::NegY, RotDir::CounterClockwise, h.cmp(&0), v.cmp(&0));
        self.fire = maybe_rot.is_some();
        if let Some(rot) = maybe_rot {
            self.rot = rot;
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

    pub fn draw(&self, pixel_size: u8, canvas: &mut Canvas, camera: WPixelVector) {

        let atlas_pos = match self.rot.matchable_8() {
            R8::D0   => TTileVector::new(1, 0),
            R8::D45  => TTileVector::new(2, 0),
            R8::D90  => TTileVector::new(2, 1),
            R8::D135 => TTileVector::new(2, 2),
            R8::D180 => TTileVector::new(1, 2),
            R8::D225 => TTileVector::new(0, 2),
            R8::D270 => TTileVector::new(0, 1),
            R8::D315 => TTileVector::new(0, 0),
        };

        pixel_draw::pixel_draw(pixel_size, canvas, PLAYER_IMAGE.get(), PixelDrawParams {
            camera,
            atlas_section: TTileRect::from_pos_dim(atlas_pos, TTileDimension::ONE).into(),
            pos: self.pos,
            z: 2,
            ..Default::default()
        });

        if self.fire {

            let (atlas_rect, rot_pivot_tile, rotation) = if let Some(rot_4) = self.rot.split_as::<4>() {
                (TTileRect::new(3, 0, 1, 2), TTileVector::new(0, 0), rot_4)
            } else {
                (TTileRect::new(4, 0, 2, 2), TTileVector::new(1, 0), (self.rot - Rot::R8_45).split_as::<4>().unwrap())
            };

            pixel_draw::pixel_draw(pixel_size, canvas, PLAYER_IMAGE.get(), PixelDrawParams {
                camera,
                atlas_section: atlas_rect.into(),
                pos: self.pos,
                rot_pivot_tile,
                rotation,
                z: 2,
            });

        }

    }

}