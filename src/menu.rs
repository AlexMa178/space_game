use ggez::Context;
use ggez::graphics::GraphicsContext;

use ggez::winit::keyboard::KeyCode;

use ggez_pixel_canvas::{ PixelCanvas, PixelDrawParams, ToPixel };
use imageable_tile_grid::{ CharTile, MultiTile, Tile as GridTile, TileGrid };

use crate::{ Pixel, ScreenPos, Tile };
use crate::sounds::AllSounds;
use crate::{ FULL_FRAME_FPS, NUM_LEVELS, SCREEN_TILES };
use crate::assets::LETTERS_IMAGE;

pub type UIGrid = TileGrid::<LetterTile, {SCREEN_TILES.width as usize}, {SCREEN_TILES.height as usize}>;

#[derive(Clone, Copy)]
pub enum LetterTile {
    BigTop { i: u8 }, BigBottom { i: u8 }, Small { i: u8 }, Digit { n: u8 }, Dot, RightArrow, LeftArrow, Colon, Point, Empty
}
impl GridTile for LetterTile {
    type TileUnit = Tile;
    fn atlas_pos(&self) -> [ u8; 2 ] {
        match self {
            Self::BigTop    { i } => [ *i, 0 ],
            Self::BigBottom { i } => [ *i, 1 ],
            Self::Small     { i } => [ *i, 2 ],
            Self::Digit     { n } => [ *n, 3 ],
            Self::Dot        => [ 10, 3 ],
            Self::RightArrow => [ 11, 3 ],
            Self::LeftArrow  => [ 12, 3 ],
            Self::Colon      => [ 13, 3 ],
            Self::Point      => [ 14, 3 ],
            Self::Empty      => [ 15, 3 ],
        }
    }
}
impl CharTile for LetterTile {
    fn from_char(c: char) -> Self {
        match c {
            '-' => Self::Dot,
            '>' => Self::RightArrow,
            '<' => Self::LeftArrow,
            ':' => Self::Colon,
            '.' => Self::Point,
            ' ' => Self::Empty,
            c if c.is_ascii_digit() => Self::Digit { n: c as u8 - b'0' },
            c if c.is_ascii_lowercase() => Self::Small { i: c as u8 - b'a' },
            c => panic!("invalid char (small): \"{c}\""),
        }
    }
}

enum BigLetter {
    Letter { i: u8 }, Space
}
impl MultiTile for BigLetter {
    type SubTile = LetterTile;
    fn dimensions(&self) -> [ u8; 2 ] {
        [ 1, 2 ]
    }
    fn sub(&self, x: u8, y: u8) -> Self::SubTile {
        match (self, x, y) {
            (Self::Letter { i }, 0, 0) => LetterTile::BigTop { i: *i },
            (Self::Letter { i }, 0, 1) => LetterTile::BigBottom { i: *i },
            (Self::Space, 0, 0 | 1) => LetterTile::Empty,
            _ => panic!("invalid sub"),
        }
    }
}
impl CharTile for BigLetter {
    fn from_char(c: char) -> Self {
        match c {
            ' ' => Self::Space,
            c if c.is_ascii_lowercase() => Self::Letter { i: c as u8 - b'a' },
            c => panic!("invalid char (big): \"{c}\""),
        }
    }
}

pub struct TitleMenu {
    selection: u8,
}
impl TitleMenu {

    pub fn new() -> Self {

        Self { selection: 0 }

    }

    pub fn update(&mut self, sounds: &mut AllSounds, ctx: &mut Context, key: KeyCode, pixel_size: &mut u8, number_subframes: &mut u8) -> bool {

        let s = &mut self.selection;
        let ps = pixel_size;
        let ns = number_subframes;

        match (*s, key) {
            (n, KeyCode::ArrowUp | KeyCode::KeyW) => {
                *s = if n == 0 { 4 } else { n - 1 };
                sounds.play_menu_navigate();
            },
            (n, KeyCode::ArrowDown | KeyCode::KeyS) => {
                *s = if n == 4 { 0 } else { n + 1 };
                sounds.play_menu_navigate();
            },
            (0, KeyCode::Enter | KeyCode::NumpadEnter) => {
                sounds.play_menu_click();
                return true;
            },
            (4, KeyCode::Enter | KeyCode::NumpadEnter) => { ctx.request_quit(); },
            (1, KeyCode::ArrowLeft | KeyCode::KeyA) => {
                match (sounds.sfx_muted(), sounds.music_muted()) {
                    (false, false) => (),
                    (false, true) => { sounds.set_music_muted(false); sounds.play_menu_click(); },
                    (true, false) => { sounds.set_sfx_muted(false); sounds.set_music_muted(true); sounds.play_menu_click(); },
                    (true, true) => { sounds.set_music_muted(false); },
                };
            },
            (1, KeyCode::ArrowRight | KeyCode::KeyD) => {
                match (sounds.sfx_muted(), sounds.music_muted()) {
                    (false, false) => { sounds.set_music_muted(true); sounds.play_menu_click(); },
                    (false, true) => { sounds.set_sfx_muted(true); sounds.set_music_muted(false); },
                    (true, false) => { sounds.set_music_muted(true); },
                    (true, true) => (),
                };
            },
            (2, KeyCode::ArrowLeft | KeyCode::KeyA) if *ps !=  1 => {
                *ps -= 1;
                let [ w, h ] = SCREEN_TILES.to_pixel().as_::<ScreenPos>().map(|s| s * *ps as f32).into();
                ctx.gfx.set_drawable_size(w, h).unwrap();
                sounds.play_menu_click();
            },
            (2, KeyCode::ArrowRight | KeyCode::KeyD) if *ps != 15 => {
                *ps += 1;
                let [ w, h ] = SCREEN_TILES.to_pixel().as_::<ScreenPos>().map(|s| s * *ps as f32).into();
                ctx.gfx.set_drawable_size(w, h).unwrap();
                sounds.play_menu_click();
            },
            (3, KeyCode::ArrowLeft | KeyCode::KeyA) if *ns !=  1 => {
                *ns -= 1;
                sounds.play_menu_click();
            },
            (3, KeyCode::ArrowRight | KeyCode::KeyD) if *ns != 12 => {
                *ns += 1;
                sounds.play_menu_click();
            },
            _ => (),
        };

        false

    }

    pub fn draw(&self, sounds: &AllSounds, pixel_size: u8, number_subframes: u8, canvas: &mut PixelCanvas, gfx: &mut GraphicsContext) {

        let s = self.selection;
        let ps = pixel_size;
        let ns = number_subframes;

        let (sound_n, sound_text) = match (sounds.sfx_muted(), sounds.music_muted()) {
            (false, false) => (0, "unmuted"),
            (false, true) => (1, "sfx only"),
            (true, false) => (2, "music only"),
            (true, true) => (3, "muted"),
        };

        let text_grid = UIGrid::fill(LetterTile::Empty)
            .builder(1, 1).write_multi::<BigLetter>("space game").set_x(20).write_multi::<BigLetter>("alex marion")
            .set_x(1).move_y(4).write(if s == 0 { "-" } else { "" }).set_x(3).write("play")
            .set_x(17).write(if s == 0 { "enter" } else { "" })
            .set_x(1).move_y(3).write(if s == 1 { "-" } else { "" }).set_x(3).write("sound")
            .set_x(16).write(if s == 1 && sound_n != 0 { "<" } else { " " }).write(sound_text).write(if s == 1 && sound_n != 3 { ">" } else { "" })
            .set_x(1).move_y(2).write(if s == 2 { "-" } else { "" }).set_x(3).write("pixel size")
            .set_x(16).write(if s == 2 && ps != 1 { "<" } else { " " }).write(ps.to_string().as_str()).write(if s == 2 && ps != 15 { ">" } else { "" })
            .set_x(1).move_y(2).write(if s == 3 { "-" } else { "" }).set_x(3).write("subframes")
            .set_x(16).write(if s == 3 && ns != 1 { "<" } else { " " }).write(ns.to_string().as_str()).write(if s == 3 && ns != 12 { ">" } else { "" })
            .set_x(1).move_y(3).write(if s == 4 { "-" } else { "" }).set_x(3).write("exit")
            .set_x(17).write(if s == 4 { "enter" } else { "" })
            .build();

        let composed = text_grid.compose_image(gfx, LETTERS_IMAGE.get()).unwrap();
        canvas.draw(&composed, PixelDrawParams::<Pixel>::default());

    }

}

pub enum LSMUpdateResponse {
    ToLevel { level_id: usize }, Back, Nothing
}

pub struct LevelSelectMenu {
    selection: u8
}
impl LevelSelectMenu {

    pub fn new(progress_len: usize) -> Self {

        Self { selection: progress_len as u8 }

    }

    pub fn update(&mut self, sounds: &AllSounds, key: KeyCode, progress_len: usize) -> LSMUpdateResponse {

        let s = &mut self.selection;
        let p = progress_len.min(NUM_LEVELS - 1) as u8;

        match key {
            KeyCode::ArrowUp | KeyCode::KeyW => {
                *s = if *s == 0 { p + 1 } else { *s - 1 };
                sounds.play_menu_navigate();
            },
            KeyCode::ArrowDown | KeyCode::KeyS => {
                *s = if *s == p + 1 { 0 } else { *s + 1 };
                sounds.play_menu_navigate();
            },
            KeyCode::Enter | KeyCode::NumpadEnter => {
                sounds.play_menu_click();
                return if *s == p + 1 {
                    LSMUpdateResponse::Back
                } else {
                    LSMUpdateResponse::ToLevel { level_id: *s as usize }
                }
            },
            _ => (),
        }

        LSMUpdateResponse::Nothing

    }

    pub fn draw(&self, canvas: &mut PixelCanvas, gfx: &mut GraphicsContext, progress: &[u64]) {

        let s = self.selection;
        let p = progress.len().min(NUM_LEVELS - 1) as u8;

        let mut text_grid = UIGrid::fill(LetterTile::Empty)
            .builder(1, 1).write_multi::<BigLetter>("level select").build();
        for i in 0..=p {
            let y = 5 + i;
            text_grid = text_grid.builder(3, y).write((i + 1).to_string().as_str()).build();
            match progress.get(i as usize) {
                Some(n) => {
                    let time_string = { let decimal = n * 10 / FULL_FRAME_FPS as u64; format!("{}.{}", decimal / 10, decimal % 10) };
                    text_grid = text_grid.builder(13 - time_string.len() as u8, y).write(&time_string).build();
                },
                None => {
                    text_grid = text_grid.builder(9, y).write("none").build();
                },
            }
        }
        if s != p + 1 { text_grid = text_grid.builder(1, 5 + s).write("-").set_x(17).write("enter").build(); }
        text_grid = text_grid.builder(1, 7 + p)
            .write(if s == p + 1 { "-" } else { "" }).set_x(3).write("back")
            .set_x(17).write(if s == p + 1 { "enter" } else { "" }).build();
        if progress.len() == NUM_LEVELS {
            let sum_string = { let decimal = progress.iter().sum::<u64>() * 10 / FULL_FRAME_FPS as u64; format!("{}.{}", decimal / 10, decimal % 10) };
            let sum_label_string = format!("sum:{sum_string}");
            text_grid = text_grid.builder(30 - sum_label_string.len() as u8, 1).write(&sum_label_string).build();
        }

        let composed = text_grid.compose_image(gfx, LETTERS_IMAGE.get()).unwrap();
        canvas.draw(&composed, PixelDrawParams::<Pixel>::default());

    }

}