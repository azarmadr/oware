use crate::GameState;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
// use bevy_kira_audio::AudioSource;

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(GameState::Loading)
                .with_collection::<FontAssets>()
                // .with_collection::<AudioAssets>()
                .with_collection::<BoardAssets>()
                .continue_to_state(GameState::Menu),
        );
    }
}

#[derive(AssetCollection, Resource)]
pub struct FontAssets {
    #[asset(path = "fonts/FiraSans-Bold.ttf")]
    pub fira_sans: Handle<Font>,
}

/*
#[derive(AssetCollection, Resource)]
pub struct AudioAssets {
    #[asset(path = "audio/flying.ogg")]
    pub flying: Handle<AudioSource>,
}
*/

#[derive(AssetCollection, Resource)]
pub struct BoardAssets {
    #[asset(path = "textures/bevy.png")]
    pub bevy: Handle<Image>,
    #[asset(path = "textures/Ghostpixxells_pixelfood/69_meatball.png")]
    pub meatball: Handle<Image>,
    #[asset(path = "textures/Ghostpixxells_pixelfood/04_bowl.png")]
    pub bowl: Handle<Image>,
    #[asset(path = "textures/Ghostpixxells_pixelfood/05_meatball_bowl.png")]
    pub meatball_bowl: Handle<Image>,
    #[asset(path = "fonts/FiraSans-Bold.ttf")]
    pub fira_sans: Handle<Font>,
}
