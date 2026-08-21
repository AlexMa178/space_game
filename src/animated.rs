use glamour::{ Point2, Rect, Size2, Vector2 };

use ggez_pixel_canvas::{ PixelCanvas, PixelDrawParams, ToPixel };

use crate::{ TILE_PIXELS, Pixel, Tile };
use crate::assets::{ BLACK_HOLE_IMAGE, EXPLOSION_IMAGE, PORTAL_COLLAPSE_IMAGE, PORTAL_IMAGE };

pub struct Explosion {
    pos: Point2<Pixel>,
    frame: u8,
    frame_counter: u8,
}
impl Explosion {

    pub fn new(obj_pos: Point2<Pixel>) -> Self {
        
        Self {
            pos: obj_pos - Vector2::<Tile>::new(1, 1).to_pixel(),
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

    pub fn draw(&self, canvas: &mut PixelCanvas, camera: Point2<Pixel>) {

        let atlas_rect = Rect::<Tile>::from_origin_and_size([ self.frame * 3, 0 ], [ 3, 3 ]);

        canvas.draw(EXPLOSION_IMAGE.get(), PixelDrawParams::default()
            .dest(self.pos - camera)
            .atlas_rect(atlas_rect.to_tuple())
            .z(3)
        );

    }

}

pub enum PortalAnimation {
    Idle { frame: u8 },
    Collapsing { frame: u8 },
    DoneCollapsing,
}

pub struct Portal {
    center_pos: Point2<Tile>,
    anim: PortalAnimation,
}
impl Portal {

    pub fn new(center_pos: Point2<Tile>) -> Self {
        Self { center_pos, anim: PortalAnimation::Idle { frame: 0 } }
    }

    pub fn rect(&self) -> Rect<Tile> {
        let n = match self.anim {
            PortalAnimation::Idle { .. }                                         => 1,
            PortalAnimation::Collapsing { .. } | PortalAnimation::DoneCollapsing => 2,
        };
        Rect::new(self.center_pos - Vector2::splat(n), Size2::splat(2 * n + 1))
    }

    pub fn collides_with(&self, obj_rect: Rect<Pixel>) -> bool {
        let obj_center = obj_rect.as_::<f32>().center();
        let self_center = self.rect().to_pixel().as_::<f32>().center();
        obj_center.distance(self_center) < TILE_PIXELS as f32 * 2.
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

    pub fn draw(&self, canvas: &mut PixelCanvas, camera: Point2<Pixel>) {

        let (image, atlas_x) = match self.anim {
            PortalAnimation::Idle { frame } => {
                (PORTAL_IMAGE.get(), frame * 3)
            },
            PortalAnimation::Collapsing { frame } => {
                (PORTAL_COLLAPSE_IMAGE.get(), frame * 5)
            },
            PortalAnimation::DoneCollapsing => {
                (PORTAL_COLLAPSE_IMAGE.get(), 10)
            }
        };

        let rect = self.rect();

        canvas.draw(image, PixelDrawParams::default()
            .dest(rect.origin.to_pixel() - camera)
            .atlas_rect(([ atlas_x, 0 ], rect.size))
            .z(3)
        );

    }

}

pub struct BlackHole {
    center_pos: Point2<Tile>,
    frame: u8,
}
impl BlackHole {

    pub fn new(center_pos: Point2<Tile>) -> Self {
        Self { center_pos, frame: 0 }
    }

    pub fn rect(&self) -> Rect<Tile> {
        Rect::new(self.center_pos - Vector2::splat(1), Size2::splat(3))
    }

    pub fn update(&mut self, subframe_counter: u8) {

        if subframe_counter == 0 {
            self.frame = if self.frame == 3 { 0 } else { self.frame + 1 };
        }

    }

    pub fn collides_with(&self, obj_rect: Rect<Pixel>) -> bool {
        let obj_center = obj_rect.as_::<f32>().center();
        let self_center = self.rect().to_pixel().as_::<f32>().center();
        obj_center.distance(self_center) < TILE_PIXELS as f32 * (3. / 2.)
    }

    pub fn influence(&self, obj_rect: Rect<Pixel>) -> Vector2<Pixel> {
        
        let self_rect = self.rect().to_pixel();

        let self_center = self_rect.as_::<f32>().center();
        let obj_center = obj_rect.as_::<f32>().center();
        let dist = self_center.distance(obj_center).round() as u64;
        let strength = 6u64.saturating_sub(dist / 12) as i32;

        let self_min = self_rect.min();
        let self_max = self_rect.max();
        let obj_min = obj_rect.min();
        let obj_max = obj_rect.max();
        let x = if obj_min.x < self_min.x { 1 } else { 0 } + if obj_max.x > self_max.x { -1 } else { 0 };
        let y = if obj_min.y < self_min.y { 1 } else { 0 } + if obj_max.y > self_max.y { -1 } else { 0 };
        Vector2::new(x * strength, y * strength)
        
    }

    pub fn draw(&self, canvas: &mut PixelCanvas, camera: Point2<Pixel>) {

        let rect = self.rect();
        let atlas_rect = Rect::new(Point2::new(self.frame * 3, 0), rect.size);

        canvas.draw(BLACK_HOLE_IMAGE.get(), PixelDrawParams::default()
            .dest(rect.origin.to_pixel() - camera)
            .atlas_rect(atlas_rect.to_tuple())
            .z(3)
        );

    }

}