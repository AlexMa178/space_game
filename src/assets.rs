use std::array;
use std::marker::PhantomData;
use std::ops::Deref;
use std::sync::{ LazyLock, OnceLock };

use ggez::audio::SoundData;
use ggez::graphics::Image;
use ggez::context::Context;

use ggez::winit::window::Icon;
use serde::de::DeserializeOwned;

use crate::{ NUM_LEVELS, GameSave };
use crate::level::{ Level, LevelCollision, LevelData };

pub static PLAYER_IMAGE         : Asset<Image> = Asset::new("/images/player.png");
pub static EXPLOSION_IMAGE      : Asset<Image> = Asset::new("/images/explosion.png");
pub static PORTAL_IMAGE         : Asset<Image> = Asset::new("/images/portal.png");
pub static PORTAL_COLLAPSE_IMAGE: Asset<Image> = Asset::new("/images/portal_collapse.png");
pub static BLACK_HOLE_IMAGE     : Asset<Image> = Asset::new("/images/black_hole.png");
pub static BACKGROUND_IMAGE     : Asset<Image> = Asset::new("/images/background.png");
pub static LETTERS_IMAGE        : Asset<Image> = Asset::new("/images/letters.png");
pub const ALL_IMAGES: [ &Asset<Image>; 7 ] = [ &PLAYER_IMAGE, &EXPLOSION_IMAGE, &PORTAL_IMAGE, &PORTAL_COLLAPSE_IMAGE, &BLACK_HOLE_IMAGE, &BACKGROUND_IMAGE, &LETTERS_IMAGE ];

pub static MENU_NAVIGATE_SFX  : Asset<SoundData> = Asset::new("/sound_effects/menu_navigate.wav");
pub static MENU_CLICK_SFX     : Asset<SoundData> = Asset::new("/sound_effects/menu_click.wav");
pub static EXPLOSION_SFX      : Asset<SoundData> = Asset::new("/sound_effects/explosion.wav");
pub static LEVEL_CLEAR_SFX    : Asset<SoundData> = Asset::new("/sound_effects/level_clear.wav");
pub static BOUNCE_SFX         : Asset<SoundData> = Asset::new("/sound_effects/bounce.wav");
pub static BLACK_HOLE_PULL_SFX: Asset<SoundData> = Asset::new("/sound_effects/black_hole_pull.wav");
pub static MENU_MUSIC         : Asset<SoundData> = Asset::new("/music/menu_music.mp3");
pub static GAME_MUSIC         : Asset<SoundData> = Asset::new("/music/game_music.mp3");
pub const ALL_SOUNDS: [ &Asset<SoundData>; 8 ] = [ &MENU_NAVIGATE_SFX, &MENU_CLICK_SFX, &EXPLOSION_SFX, &LEVEL_CLEAR_SFX, &BOUNCE_SFX, &BLACK_HOLE_PULL_SFX, &MENU_MUSIC, &GAME_MUSIC ];

pub static ICON: Asset<Icon> = Asset::new("/images/icon.png");

pub static INITIAL_GAME_SAVE: Asset<Ron<GameSave>> = Asset::new("/game_save.ron");

pub static LEVELS: LazyLock<[ Asset<Level>; NUM_LEVELS ]> = LazyLock::new(|| array::from_fn(|i| Asset::new(format!("/levels/level_{i}").leak())));

pub fn load_all_assets(ctx: &Context) {
    ALL_IMAGES.into_iter().for_each(|image| image.load(ctx));
    ALL_SOUNDS.into_iter().for_each(|sound| sound.load(ctx));
    ICON.load(ctx);
    INITIAL_GAME_SAVE.load(ctx);
    LEVELS.iter().for_each(|level| level.load(ctx));
}

pub struct Asset<T: AssetType> {
    value: OnceLock<T::ValueType>,
    path: &'static str,
}
impl<T: AssetType> Asset<T> {
    pub const fn new(path: &'static str) -> Self {
        Self { value: OnceLock::new(), path }
    }
    pub fn load(&self, ctx: &Context) {
        self.value.get_or_init(|| T::load(ctx, self.path));
    }
    pub fn get(&self) -> &T::ValueType {
        self.value.get().unwrap()
    }
}
impl <T: AssetType<ValueType: Clone>> Asset<T> {
    pub fn get_cloned(&self) -> T::ValueType {
        self.get().clone()
    }
}
impl<T: AssetType> Deref for Asset<T> {
    type Target = T::ValueType;
    fn deref(&self) -> &T::ValueType {
        self.get()
    }
}

pub trait AssetType {
    type ValueType;
    fn load(ctx: &Context, path: &str) -> Self::ValueType;
}

impl AssetType for Image {
    type ValueType = Self;
    fn load(ctx: &Context, path: &str) -> Self {
        Image::from_path(ctx, path).unwrap()
    }
}
impl AssetType for SoundData {
    type ValueType = Self;
    fn load(ctx: &Context, path: &str) -> SoundData {
        SoundData::new(ctx, path).unwrap()
    }
}
impl AssetType for Icon {
    type ValueType = Self;
    fn load(ctx: &Context, path: &str) -> Icon {
        let image = Image::from_path(ctx, path).unwrap();
        Icon::from_rgba(image.to_pixels(ctx).unwrap(), image.width(), image.height()).unwrap()
    }
}
pub struct Ron<T> { phantom: PhantomData<T> }
impl<T: DeserializeOwned> AssetType for Ron<T> {
    type ValueType = T;
    fn load(ctx: &Context, path: &str) -> T {
        ron::from_str(&ctx.fs.read_to_string(path).unwrap()).unwrap()
    }
}
impl AssetType for LevelCollision {
    type ValueType = Self;
    fn load(ctx: &Context, path: &str) -> Self::ValueType {
        LevelCollision::from_csv_text(&ctx.fs.read_to_string(path).unwrap())
    }
}
impl AssetType for Level {
    type ValueType = Self;
    fn load(ctx: &Context, path: &str) -> Self {
        let tiles = Image::load(ctx, &format!("{path}/tiles.png"));
        let collision = LevelCollision::load(ctx, &format!("{path}/collision.csv"));
        let data = Ron::<LevelData>::load(ctx, &format!("{path}/data.ron"));
        Level::from_file_parts(tiles, collision, data)
    }
}