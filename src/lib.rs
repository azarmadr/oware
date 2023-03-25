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
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use self::tweens::GameTweeningPlugin;

#[derive(Clone,Default,States, Eq, PartialEq, Debug, Hash, Copy)]
enum GameState {
    #[default]
    Loading,
    Game,
    Menu,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_state::<GameState>()
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
            .add_system(auto_start)
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
fn auto_start(
    mut act: EventWriter<crate::menu::Actions>,
    time: Res<Time>,
    mut timer: Local<Timer>,
) {
    use self::menu::Actions;
    use std::time::Duration;

    if timer.duration() == Duration::ZERO {
        timer.set_duration(Duration::from_millis(729));
    }
    if timer.tick(time.delta()).just_finished() {
        act.send(Actions::NewGame);
    }
}

/// Despawn all entities with a given component type
fn despawn_with<T: Component>(mut commands: Commands, q: Query<Entity, With<T>>) {
    for e in q.iter() {
        commands.entity(e).despawn_recursive();
    }
}
