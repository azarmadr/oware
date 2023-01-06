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

fn sprite(texture: Handle<Image>, size: Option<f32>, transform: Transform) -> SpriteBundle {
    SpriteBundle {
        texture,
        sprite: Sprite {
            custom_size: size.map(|x| Vec2::new(x, x)),
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
    pub fn meatball_sprite(&self, size: f32, transform: Transform) -> SpriteBundle {
        sprite(self.meatball.clone(), Some(size), transform)
    }
    pub fn meatball_bowl_sprite(&self, size: f32, transform: Transform) -> SpriteBundle {
        sprite(self.meatball_bowl.clone(), Some(size), transform)
    }
    pub fn bowl_sprite(&self, size: f32, transform: Transform) -> SpriteBundle {
        sprite(self.bowl.clone(), Some(size), transform)
    }
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
            .with_alignment(TextAlignment::CENTER),
            transform,
            ..default()
        }
    }
}
