#![windows_subsystem = "windows"]

mod assets;
mod player;
mod animated;
mod camera;
mod level;
mod menu;
mod sounds;

use std::collections::HashSet;
use std::io::Write;
use std::path::PathBuf;
use std::env;

use ggez::context::{ Context, ContextBuilder };
use ggez::event::{ self, EventHandler };
use ggez::error::{ GameError, GameResult };
use ggez::conf::WindowSetup;
use ggez::graphics::{Canvas, Color, DrawParam, Sampler};
use ggez::input::keyboard::KeyInput;

use ggez::winit::keyboard::{ KeyCode, PhysicalKey };
use ggez::winit::platform::windows::{ CornerPreference, WindowExtWindows };
use ggez::winit::dpi::PhysicalPosition;
use ggez::winit::window::WindowButtons;

use glamour::{ Point2, Size2, Unit, Vector2 };

use serde::{ Deserialize, Serialize };

use ggez_pixel_canvas::{ AsPixel, PixelCanvas, PixelDrawParams, ToPixel };

use crate::assets::{ BACKGROUND_IMAGE, ICON, INITIAL_GAME_SAVE, LETTERS_IMAGE, LEVELS };
use crate::player::Player;
use crate::animated::{ Explosion, Portal, BlackHole };
use crate::level::CollisionType;
use crate::menu::{ LSMUpdateResponse, LetterTile, LevelSelectMenu, TitleMenu, UIGrid };
use crate::sounds::AllSounds;

pub const FULL_FRAME_FPS: u32 = 5;
pub const TILE_PIXELS: i32 = 8;
pub const SCREEN_TILES: Size2<Tile> = Size2::new(32, 18);
pub const NUM_LEVELS: usize = 8;

pub struct Tile;
impl Unit for Tile {
    type Scalar = u8;
}
impl AsPixel for Tile {
    type PixelType = Pixel;
    const SIZE: i32 = TILE_PIXELS;
}

pub struct Pixel;
impl Unit for Pixel {
    type Scalar = i32;
}
impl AsPixel for Pixel {
    type PixelType = Self;
    const SIZE: i32 = 1;
}

pub struct ScreenPos;
impl Unit for ScreenPos {
    type Scalar = f32;
}

fn main() -> GameResult {

    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        PathBuf::from("./resources")
    };

    let (mut ctx, event_loop) = ContextBuilder::new("space_game", "2026")
        .window_setup(WindowSetup::default().title("space_game"))
        .add_resource_path(resource_dir)
        .build()?;

    if !ctx.fs.exists("/game_save.ron") {
        ctx.fs.create("/game_save.ron")?.write_all(ron::to_string(&GameSave::NONEXISTENT_DEFAULT).unwrap().as_bytes()).unwrap();
    }
    assets::load_all_assets(&ctx);

    ctx.gfx.window().set_enabled_buttons(WindowButtons::all() & !WindowButtons::MAXIMIZE);
    ctx.gfx.window().set_corner_preference(CornerPreference::DoNotRound);
    ctx.gfx.window().set_window_icon(Some(ICON.get_cloned()));
    ctx.gfx.window().set_taskbar_icon(Some(ICON.get_cloned()));
    let [ window_w, window_h ] = SCREEN_TILES.to_pixel().as_::<ScreenPos>().map(|s| s * INITIAL_GAME_SAVE.get().pixel_size as f32).into();
    ctx.gfx.set_drawable_size(window_w, window_h)?;
    let game = Game::new(&ctx);
    event::run(ctx, event_loop, game)

}

enum PlayerExistence {
    Exists {
        player: Player,
        timer: u64,
    },
    Wait { wait_counter: u8 },
    LevelEnded {
        final_time: u64,
    },
}

struct PlayingState {
    maybe_player: PlayerExistence,
    maybe_explosion: Option<Explosion>,
    portal: Portal,
    black_holes: Vec<BlackHole>,
    level_id: usize,
    camera: Point2<Pixel>,
    subframe_counter: u8,
}
impl PlayingState {

    fn new(level_id: usize) -> Self {
        let level = LEVELS[ level_id ].get();
        Self {
            maybe_player: PlayerExistence::Wait { wait_counter: 4 },
            maybe_explosion: None,
            portal: Portal::new(level.spawn_portal_pos),
            black_holes: level.spawn_black_holes_pos.iter().map(|pos| BlackHole::new(*pos)).collect(),
            level_id,
            camera: camera::initial(level),
            subframe_counter: 0,
        }
    }

}

enum State {
    TitleMenu { menu: TitleMenu },
    LevelSelectMenu { menu: LevelSelectMenu },
    Playing { state: PlayingState },
}
impl Default for State {
    fn default() -> Self {
        Self::TitleMenu { menu: TitleMenu::new() }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct GameSave {
    sfx_muted: bool,
    music_muted: bool,
    pixel_size: u8,
    num_subframes: u8,
    progress: Vec<u64>,
}
impl GameSave {

    pub const NONEXISTENT_DEFAULT: Self = Self { sfx_muted: false, music_muted: false, pixel_size: 6, num_subframes: 8, progress: Vec::new() };

}

struct Game {
    state: State,
    sounds: AllSounds,
    pixel_size: u8,
    num_subframes: u8,
    progress: Vec<u64>,
}
impl Game {

    pub fn new(ctx: &Context) -> Self {

        let GameSave { sfx_muted, music_muted, pixel_size, num_subframes, progress } = INITIAL_GAME_SAVE.get();

        Self {
            state: State::default(),
            sounds: AllSounds::new(&ctx.audio, *music_muted, *sfx_muted),
            pixel_size: *pixel_size,
            num_subframes: *num_subframes,
            progress: progress.clone(),
        }
    }

}
impl EventHandler for Game {

    fn quit_event(&mut self, ctx: &mut Context) -> Result<bool, GameError> {

        let final_game_save = GameSave {
            sfx_muted: self.sounds.sfx_muted(),
            music_muted: self.sounds.music_muted(),
            pixel_size: self.pixel_size,
            num_subframes: self.num_subframes,
            progress: self.progress.clone(),
        };
        ctx.fs.create("/game_save.ron")?.write_all(ron::to_string(&final_game_save).unwrap().as_bytes()).unwrap();
        Ok(false)

    }

    fn resize_event(&mut self, ctx: &mut Context, width: f32, height: f32) -> GameResult {
        
        let [ monitor_w, monitor_h ]: [f32; 2] = ctx.gfx.window().current_monitor().unwrap().size().into();
        let window_pos = [ monitor_w / 2. - width / 2., monitor_h / 2. - height / 2. ];
        ctx.gfx.set_window_position(PhysicalPosition::<f32>::from(window_pos))?;
        Ok(())

    }

    fn key_down_event(&mut self, ctx: &mut Context, input: KeyInput, repeated: bool) -> GameResult {

        if repeated { return Ok(()); }

        let PhysicalKey::Code(key_code) = input.event.physical_key else { return Ok(()); };

        let Game { state, sounds, pixel_size, num_subframes, progress } = self;
        match state {
            State::Playing { state: PlayingState { level_id, .. } } => {

                if key_code == KeyCode::KeyR {
                    *state = State::Playing { state: PlayingState::new(*level_id) }
                }
                if key_code == KeyCode::Escape {
                    *state = State::TitleMenu { menu: TitleMenu::new() };
                    sounds.set_to_menu();
                }

            },
            State::TitleMenu { menu } => {

                if input.mods.alt_key() && input.mods.shift_key() && key_code == KeyCode::Delete {
                    let GameSave { sfx_muted: ifm, music_muted: imm, pixel_size: ips, num_subframes: ins, progress: ip } = GameSave::NONEXISTENT_DEFAULT;
                    sounds.set_sfx_muted(ifm);
                    sounds.set_music_muted(imm);
                    *pixel_size = ips;
                    *num_subframes = ins;
                    *progress = ip;
                    ctx.request_quit();
                }

                let to_game = menu.update(sounds, ctx, key_code, pixel_size, num_subframes);
                if to_game {
                    *state = if progress.is_empty() {
                        sounds.set_to_game();
                        State::Playing { state: PlayingState::new(0) }
                    } else {
                        State::LevelSelectMenu { menu: LevelSelectMenu::new(progress.len()) }
                    };
                }

            },
            State::LevelSelectMenu { menu } => {
                
                let response = menu.update(sounds, key_code, progress.len());
                match response {
                    LSMUpdateResponse::ToLevel { level_id } => {
                        *state = State::Playing { state: PlayingState::new(level_id) };
                        sounds.set_to_game();
                    },
                    LSMUpdateResponse::Back => {
                        *state = State::TitleMenu { menu: TitleMenu::new() };
                    },
                    LSMUpdateResponse::Nothing => (),
                }

            }
        }
        Ok(())

    }

    fn update(&mut self, ctx: &mut Context) -> GameResult {

        let ns = self.num_subframes;
        
        if !ctx.time.check_update_time(FULL_FRAME_FPS * ns as u32) { return Ok(()); }

        let mut maybe_next_state = None;
        
        if let State::Playing { state: PlayingState { maybe_player, maybe_explosion, portal, black_holes, level_id, camera, subframe_counter } } = &mut self.state {

            let level = LEVELS[ *level_id ].get();

            if let Some(explosion) = maybe_explosion {
                let expired = explosion.update(ns);
                if expired { *maybe_explosion = None; }
            }

            if portal.update(*subframe_counter) {
                maybe_next_state = Some(State::LevelSelectMenu { menu: LevelSelectMenu::new(self.progress.len()) });
                self.sounds.set_to_menu();
            };

            for black_hole in black_holes.iter_mut() {
                black_hole.update(*subframe_counter);
            }

            match maybe_player {
                PlayerExistence::Exists { player, timer } => {

                    *camera = camera::find_camera_pos(player.pos(), player.vel(), level.collision.dim());

                    let keys = ctx.keyboard.pressed_physical_keys.iter().filter_map(|pk| if let PhysicalKey::Code(key_code) = pk { Some(*key_code) } else { None }).collect::<HashSet<_>>();
                    player.update(&mut self.sounds, *subframe_counter, ns, keys, &level.collision, black_holes);

                    let player_pos = player.pos();
                    let player_rect = player.rect();

                    if portal.collides_with(player_rect) {
                        self.sounds.play_level_clear();
                        match self.progress.get(*level_id) {
                            Some(previous_best) => self.progress[*level_id] = *previous_best.min(timer),
                            None => self.progress.push(*timer),
                        };
                        *maybe_player = PlayerExistence::LevelEnded { final_time: *timer };
                        portal.set_collapsing();
                    }

                    if level.collision.touching(player_rect).contains(&CollisionType::Wall)
                        || black_holes.iter().any(|b| b.collides_with(player_rect)) {
                        
                        self.sounds.play_explosion();
                        *maybe_explosion = Some(Explosion::new(player_pos));
                        *maybe_player = PlayerExistence::Wait { wait_counter: 4 };

                    }

                },
                PlayerExistence::Wait { wait_counter } => {

                    if *subframe_counter == 0 {
                        if *wait_counter == 0 {
                            *maybe_player = PlayerExistence::Exists { player: Player::new(level.spawn_player_pos), timer: 0 };
                            *camera = camera::initial(level);
                        } else {
                            *wait_counter -= 1;
                        };
                    }

                },
                PlayerExistence::LevelEnded { .. } => (),
            }

            *subframe_counter += 1;
            if *subframe_counter == ns {
                *subframe_counter = 0;
                if let PlayerExistence::Exists { timer, .. } = maybe_player {
                    *timer += 1;
                }
            }

        }

        if let Some(next_state) = maybe_next_state {
            self.state = next_state;
        }

        Ok(())

    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {

        let ns = self.num_subframes;

        let mut pixel_canvas = PixelCanvas::new::<Tile>(ctx, SCREEN_TILES);

        pixel_canvas.draw(BACKGROUND_IMAGE.get(), PixelDrawParams::<Pixel>::default());

        match &self.state {
            State::TitleMenu { menu } => {

                menu.draw(&self.sounds, self.pixel_size, ns, &mut pixel_canvas, &mut ctx.gfx);

            },
            State::LevelSelectMenu { menu } => {

                menu.draw(&mut pixel_canvas, &mut ctx.gfx, &self.progress);

            }
            State::Playing { state: PlayingState { maybe_player, maybe_explosion, portal, black_holes, level_id, camera, .. } } => {
                
                let level = LEVELS[ *level_id ].get();

                level.draw(&mut pixel_canvas, *camera);
                if let PlayerExistence::Exists { player, .. } = maybe_player {
                    player.draw(&mut pixel_canvas, *camera);
                }
                portal.draw(&mut pixel_canvas, *camera);
                if let Some(explosion) = maybe_explosion {
                    explosion.draw(&mut pixel_canvas, *camera);
                }
                for black_hole in black_holes {
                    black_hole.draw(&mut pixel_canvas, *camera);
                }

                if let Some(time) = { match maybe_player {
                    PlayerExistence::Exists { timer, .. } => Some(timer),
                    PlayerExistence::Wait { .. } => None,
                    PlayerExistence::LevelEnded { final_time } => Some(final_time),
                } } {
                    
                    let time_string = { let decimal = time * 10 / FULL_FRAME_FPS as u64; format!("{}.{}", decimal / 10, decimal % 10) };
                    let text_grid = UIGrid::fill(LetterTile::Empty).builder(31 - time_string.len() as u8, 1).write(&time_string).build();
                    let composed = text_grid.compose_image(ctx, LETTERS_IMAGE.get())?;
                    pixel_canvas.draw::<Pixel>(&composed, PixelDrawParams { z: 4, ..Default::default() });

                };

            },
        }

        let pixel_output = pixel_canvas.finish(ctx)?;

        let mut canvas = Canvas::from_frame(ctx, Color::BLACK);
        canvas.set_sampler(Sampler::nearest_clamp());
        canvas.draw(&pixel_output, DrawParam::default().scale(Vector2::<u8>::splat(self.pixel_size).as_::<f32>().to_array()));
        canvas.finish(ctx)

    }

}