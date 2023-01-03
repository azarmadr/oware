// mod actions;
// mod audio;
mod loading;
mod menu;
mod oware;
// mod player;
mod tweens;

use bevy::app::App;
use bevy::prelude::*;

// use actions::ActionsPlugin;
// use audio::InternalAudioPlugin;
use loading::LoadingPlugin;
use menu::MenuPlugin;
use oware::OwarePlugin;
// use player::PlayerPlugin;

#[cfg(debug_assertions)]
use bevy::diagnostic::LogDiagnosticsPlugin;

#[cfg(feature = "dev")]
use bevy_inspector_egui::WorldInspectorPlugin;

use self::tweens::GameTweeningPlugin;

// This example game uses States to separate logic
// See https://bevy-cheatbook.github.io/programming/states.html
// Or https://github.com/bevyengine/bevy/blob/main/examples/ecs/state.rs
#[derive(Clone, Eq, PartialEq, Debug, Hash, Copy)]
enum GameState {
    // During the loading State the LoadingPlugin will load our assets
    Loading,
    // During this State the actual game logic is executed
    Playing,
    // Here the menu is drawn and waiting for player interaction
    Menu,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_state(GameState::Loading)
            .add_plugin(LoadingPlugin)
            .add_plugin(MenuPlugin)
            .add_plugin(GameTweeningPlugin)
            // .add_plugin(ActionsPlugin)
            // .add_plugin(InternalAudioPlugin)
            // .add_plugin(PlayerPlugin);
            ;

        #[cfg(not(feature = "dev"))]
        app.add_plugin(OwarePlugin::<6>);

        #[cfg(feature = "dev")]
        app.add_plugin(OwarePlugin::<4>)
            // .add_system(auto_start)
            .add_plugin(WorldInspectorPlugin::new());

        #[cfg(debug_assertions)]
        {
            app
                // .add_plugin(FrameTimeDiagnosticsPlugin::default())
                .add_plugin(LogDiagnosticsPlugin::default());
        }
    }
}

#[cfg(feature = "dev")]
fn auto_start(mut state: ResMut<State<GameState>>, time: Res<Time>, mut timer: Local<Timer>) {
    if state.current() == &GameState::Menu {
        use std::time::Duration;

        if timer.duration() == Duration::ZERO {
            timer.set_duration(Duration::from_millis(7729));
        }
        if timer.tick(time.delta()).just_finished() {
            state.set(GameState::Playing).unwrap();
        }
    }
}
