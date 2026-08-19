use std::time::Duration;

use ggez::audio::{ AudioContext, SoundSource, Source };

use crate::assets::{ BLACK_HOLE_PULL_SFX, BOUNCE_SFX, EXPLOSION_SFX, GAME_MUSIC, LEVEL_CLEAR_SFX, MENU_CLICK_SFX, MENU_MUSIC, MENU_NAVIGATE_SFX };

#[derive(Clone, Copy, PartialEq, Eq)]
enum Music {
    Menu, Game
}

pub struct AllSounds {
    current_music: Music,
    music_muted: bool,
    sfx_muted: bool,
    menu_source: Source,
    game_source: Source,
    menu_navigate: Source,
    menu_click: Source,
    explosion: Source,
    level_clear: Source,
    bounce: Source,
    black_hole_pull: Source,
}
impl AllSounds {

    pub fn new(audio: &AudioContext, music_muted: bool, sfx_muted: bool) -> Self {

        let mut menu = Source::from_data(audio, MENU_MUSIC.get_cloned()).unwrap();
        let mut game = Source::from_data(audio, GAME_MUSIC.get_cloned()).unwrap();
        menu.set_repeat(true);
        game.set_repeat(true);
        if !music_muted {
            menu.play();
        }
        Self {
            current_music: Music::Menu,
            music_muted,
            sfx_muted,
            menu_source: menu,
            game_source: game,
            menu_navigate: Source::from_data(audio, MENU_NAVIGATE_SFX.get_cloned()).unwrap(),
            menu_click: Source::from_data(audio, MENU_CLICK_SFX.get_cloned()).unwrap(),
            explosion: Source::from_data(audio, EXPLOSION_SFX.get_cloned()).unwrap(),
            level_clear: Source::from_data(audio, LEVEL_CLEAR_SFX.get_cloned()).unwrap(),
            bounce: Source::from_data(audio, BOUNCE_SFX.get_cloned()).unwrap(),
            black_hole_pull: Source::from_data(audio, BLACK_HOLE_PULL_SFX.get_cloned()).unwrap(),
        }

    }

    fn current_music_source(&self) -> &Source {
        match self.current_music {
            Music::Menu => &self.menu_source,
            Music::Game => &self.game_source,
        }
    }

    fn change_music(&mut self, new_music: Music) {

        if self.current_music == new_music { return; };
        if !self.music_muted {
            self.current_music_source().stop();
        }
        self.current_music = new_music;
        if !self.music_muted {
            self.current_music_source().play();
        }

    }

    pub fn music_muted(&self) -> bool {
        self.music_muted
    }

    pub fn set_music_muted(&mut self, muted: bool) {

        self.music_muted = muted;
        match muted {
            true  => self.current_music_source().stop(),
            false => self.current_music_source().play(),
        }

    }

    pub fn sfx_muted(&self) -> bool {
        self.sfx_muted
    }

    pub fn set_sfx_muted(&mut self, muted: bool) {
        self.sfx_muted = muted;
    }

    pub fn set_to_menu(&mut self) {
        self.change_music(Music::Menu);
    }

    pub fn set_to_game(&mut self) {
        self.change_music(Music::Game);
    }

    pub fn play_menu_navigate(&self) {
        if !self.sfx_muted { self.menu_navigate.play(); }
    }

    pub fn play_menu_click(&self) {
        if !self.sfx_muted { self.menu_click.play(); }
    }

    pub fn play_explosion(&self) {
        if !self.sfx_muted { self.explosion.play(); }
    }

    pub fn play_level_clear(&self) {
        if !self.sfx_muted { self.level_clear.play(); }
    }

    pub fn play_bounce(&self) {
        if !self.sfx_muted && (!self.bounce.playing() || self.bounce.elapsed() > Duration::from_millis(100)) { self.bounce.play(); }
    }

    pub fn play_black_hole_pull(&mut self, volume: f32) {
        if !self.sfx_muted { self.black_hole_pull.set_volume(volume); self.black_hole_pull.play(); }
    }

}