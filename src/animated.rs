use ggez_pixel_canvas::{ PDPTileBuilder, PixelCanvas };

use crate::TILE_PIXELS;
use crate::scale::{ DPixelVector, DTileVector, PositionAndDimension, TTileRect, ToPixel, WPixelRect, WPixelVector, WTileVector };
use crate::assets::{BLACK_HOLE_IMAGE, EXPLOSION_IMAGE, PORTAL_COLLAPSE_IMAGE, PORTAL_IMAGE};

pub struct Explosion {
    pos: WPixelVector,
    frame: u8,
    frame_counter: u8,
}
impl Explosion {

    pub fn new(obj_pos: WPixelVector) -> Self {
        
        Self {
            pos: obj_pos - DTileVector::new(1, 1).to_pixel(),
            frame: 0,
            frame_counter: 0,
        }

    }

    pub fn update(&mut self, num_subframes: u8) -> bool {

        self.frame_counter += 1;
        let done = self.frame == 3;
        if self.frame_counter == num_subframes && !done {
            self.frame_counter = 0;
            self.frame += 1;
        }

        done

    }

    pub fn draw(&self, canvas: &mut PixelCanvas, camera: WPixelVector) {

        let atlas_rect = TTileRect::new(self.frame * 3, 0, 3, 3);

        canvas.draw(EXPLOSION_IMAGE.get(), PDPTileBuilder::<i32, u8>::new(TILE_PIXELS)
            .pixel_dest(self.pos - camera)
            .tile_atlas_rect(atlas_rect)
            .z(3)
        .build());

    }

}

pub enum PortalAnimation {
    Idle { frame: u8 },
    Collapsing { frame: u8 },
    DoneCollapsing,
}

pub struct Portal {
    center_pos: WTileVector,
    anim: PortalAnimation,
}
impl Portal {

    pub fn new(center_pos: WTileVector) -> Self {
        Self { center_pos, anim: PortalAnimation::Idle { frame: 0 } }
    }

    pub fn collides_with(&self, obj_pos: WPixelVector) -> bool {
        let half = TILE_PIXELS as f32 / 2.;
        let obj_center = obj_pos.as_vec2() + half;
        let portal_center = self.center_pos.to_pixel().as_vec2() + half;
        obj_center.distance(portal_center) < TILE_PIXELS as f32 * 2.
    }

    pub fn set_collapsing(&mut self) {
        self.anim = PortalAnimation::Collapsing { frame: 0 };
    }

    pub fn update(&mut self, subframe_counter: u8) -> bool {

        let mut maybe_next_anim = None;

        if subframe_counter == 0 {
            match &mut self.anim {
                PortalAnimation::Idle { frame } => {
                    *frame = if *frame == 3 { 0 } else { *frame + 1 };
                },
                PortalAnimation::Collapsing { frame } => {
                    *frame += 1;
                    if *frame == 2 {
                        maybe_next_anim = Some(PortalAnimation::DoneCollapsing);
                    }
                },
                PortalAnimation::DoneCollapsing => return true,
            }
        };

        if let Some(next_anim) = maybe_next_anim {
            self.anim = next_anim;
        }

        false

    }

    pub fn draw(&self, canvas: &mut PixelCanvas, camera: WPixelVector) {

        let (image, atlas_rect, pos) = match self.anim {
            PortalAnimation::Idle { frame } => {
                (PORTAL_IMAGE.get(), TTileRect::new(frame * 3, 0, 3, 3), self.center_pos - 1)
            },
            PortalAnimation::Collapsing { frame } => {
                (PORTAL_COLLAPSE_IMAGE.get(), TTileRect::new(frame * 5, 0, 5, 5), self.center_pos - 2)
            },
            PortalAnimation::DoneCollapsing => {
                (PORTAL_COLLAPSE_IMAGE.get(), TTileRect::new(10, 0, 5, 5), self.center_pos - 2)
            }
        };

        canvas.draw(image, PDPTileBuilder::<i32, u8>::new(TILE_PIXELS)
            .pixel_dest(pos.to_pixel() - camera)
            .tile_atlas_rect(atlas_rect)
            .z(3)
        .build());

    }

}

pub struct BlackHole {
    center_pos: WTileVector,
    frame: u8,
}
impl BlackHole {

    pub fn new(center_pos: WTileVector) -> Self {
        Self { center_pos, frame: 0 }
    }

    pub fn update(&mut self, subframe_counter: u8) {

        if subframe_counter == 0 {
            self.frame = if self.frame == 3 { 0 } else { self.frame + 1 };
        }

    }

    pub fn collides_with(&self, obj_pos: WPixelVector) -> bool {
        let half = TILE_PIXELS as f32 / 2.;
        let obj_center = obj_pos.as_vec2() + half;
        let portal_center = self.center_pos.to_pixel().as_vec2() + half;
        obj_center.distance(portal_center) < half * 3.
    }

    pub fn influence(&self, obj_rect: WPixelRect) -> DPixelVector {
        
        let self_center = self.center_pos.to_pixel().as_vec2() + TILE_PIXELS as f32 / 2.;
        let obj_center = obj_rect.pos().as_vec2() + obj_rect.dim().as_vec2() / 2.;
        let dist = self_center.distance(obj_center).round() as u64;
        let strength = 6u64.saturating_sub(dist / 12) as i32;

        let self_min = (self.center_pos - 1).to_pixel();
        let self_max = (self.center_pos + 2).to_pixel();
        let obj_min = obj_rect.pos();
        let obj_max = obj_rect.pos() + obj_rect.dim();
        let x = if obj_min.x < self_min.x { 1 } else { 0 } + if obj_max.x > self_max.x { -1 } else { 0 };
        let y = if obj_min.y < self_min.y { 1 } else { 0 } + if obj_max.y > self_max.y { -1 } else { 0 };
        DPixelVector::new(x * strength, y * strength)
        
    }

    pub fn draw(&self, canvas: &mut PixelCanvas, camera: WPixelVector) {

        let atlas_rect = TTileRect::new(self.frame * 3, 0, 3, 3);

        canvas.draw(BLACK_HOLE_IMAGE.get(), PDPTileBuilder::<i32, u8>::new(TILE_PIXELS)
            .pixel_dest((self.center_pos - 1).to_pixel() - camera)
            .tile_atlas_rect(atlas_rect)
            .z(3)
        .build());

    }

}