use crate::GameState;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

// use bevy_kira_audio::AudioSource;

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(GameState::Loading)
                .continue_to_state(GameState::Menu),
        )
            .add_collection_to_loading_state::<_,FontAssets>(GameState::Loading)
            // .add_collection_to_loading_state::<_,AudioAssets>(GameState::Loading)
            .add_collection_to_loading_state::<_,BoardAssets>(GameState::Loading)
            ;
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

pub fn sprite(texture: &Handle<Image>, size: f32, transform: Transform) -> SpriteBundle {
    SpriteBundle {
        texture: texture.clone(),
        sprite: Sprite {
            custom_size: Some(size).map(|x| Vec2::new(x, x)),
            ..default()
        },
        transform,
        ..default()
    }
}

#[derive(AssetCollection, Resource)]
pub struct BoardAssets {
    #[asset(path = "textures/bevy.png")]
    pub bevy: Handle<Image>,
    #[asset(path = "textures/Ghostpixxells_pixelfood/meatball.png")]
    pub meatball: Handle<Image>,
    #[asset(path = "textures/Ghostpixxells_pixelfood/04_bowl.png")]
    pub bowl: Handle<Image>,
    #[asset(path = "textures/Ghostpixxells_pixelfood/05_meatball_bowl.png")]
    pub meatball_bowl: Handle<Image>,
    #[asset(path = "fonts/FiraSans-Bold.ttf")]
    pub fira_sans: Handle<Font>,
}

impl BoardAssets {
    pub fn text<S: Into<String>>(
        &self,
        text: S,
        size: f32,
        color: Color,
        transform: Transform,
    ) -> Text2dBundle {
        Text2dBundle {
            text: Text::from_section(
                text,
                TextStyle {
                    font: self.fira_sans.clone(),
                    font_size: size,
                    color,
                },
            )
            .with_alignment(TextAlignment::Center),
            transform,
            ..default()
        }
    }
}
