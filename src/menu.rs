use crate::{
    despawn_with,
    oware::{Actor, Ai, PC},
    GameState,
};
#[cfg(not(target_arch = "wasm32"))]
use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_quickmenu::{style::Stylesheet, *};
use board_game::board::{Outcome, Player};
use board_game::pov::NonPov;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
enum Screens {
    Game,
    Pause,
    NewGame,
    GameOver,
    Seeds,
}
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Actions {
    Resume,
    Pause,
    #[cfg(not(target_arch = "wasm32"))]
    Quit,
    NewGame,
    PlayerAsFirst,
    Bot(Ai),
    SetSeeds(u8),
}
impl ActionTrait for Actions {
    type State = OwareCfg;
    type Event = Self;
    fn handle(&self, state: &mut Self::State, event_writer: &mut EventWriter<Self::Event>) {
        match self {
            Self::Pause | Self::Resume => event_writer.send(*self),
            #[cfg(not(target_arch = "wasm32"))]
            Self::Quit => event_writer.send(*self),
            Self::NewGame => {
                state.new_game = true;
                event_writer.send(*self)
            }
            Self::PlayerAsFirst => state.human_is_first ^= true,
            Self::Bot(ai) => state.ai = *ai,
            Self::SetSeeds(n) => state.init_seeds = *n,
        }
    }
}
impl ScreenTrait for Screens {
    type Action = Actions;
    type State = OwareCfg;
    fn resolve(
        &self,
        state: &<<Self as ScreenTrait>::Action as bevy_quickmenu::ActionTrait>::State,
    ) -> bevy_quickmenu::Menu<Self> {
        let seed_actions =
            |n| MenuItem::action(format!("{n}"), Actions::SetSeeds(n)).checked(state.init_seeds == n);
        let bot_list = &mut [
            Ai::Random,
            Ai::Rollout(27),
            Ai::Rollout(729),
            Ai::Mcts(27, 1),
            Ai::Mcts(729, 2),
        ]
        .iter()
        .map(|&x| MenuItem::action(format!("{x:?}"), Actions::Bot(x)).checked(state.ai == x))
        .collect::<Vec<MenuItem<Screens>>>();
        Menu::new(
            format!("{self:?}"),
            match self {
                Self::Pause => vec![
                    MenuItem::headline("Paused"),
                    MenuItem::action("Resume", Actions::Resume),
                    MenuItem::screen("New Game", Screens::NewGame),
                    #[cfg(not(target_arch = "wasm32"))]
                    MenuItem::action("Quit", Actions::Quit),
                ],
                Self::Game => vec![MenuItem::action("Pause", Actions::Pause)],
                Self::GameOver => vec![
                    MenuItem::headline(state.outcome()),
                    MenuItem::screen("New Game", Screens::NewGame),
                    #[cfg(not(target_arch = "wasm32"))]
                    MenuItem::action("Quit", Actions::Quit),
                ],
                Self::NewGame => {
                    let mut items = vec![
                        MenuItem::headline("Oware"),
                        MenuItem::action("Start a New Game", Actions::NewGame),
                        MenuItem::label("Configuration"),
                        MenuItem::label("Player Position"),
                        MenuItem::action("Is First", Actions::PlayerAsFirst)
                            .checked(state.human_is_first),
                        MenuItem::screen("Initial Seeds", Screens::Seeds),
                        MenuItem::label("Bot Type"),
                    ];
                    items.append(bot_list);
                    items
                },
                Self::Seeds => [MenuItem::headline("Initial Seeds")]
                    .into_iter()
                    .chain((3..6).map(|x| seed_actions(x)))
                    .collect(),
            },
        )
    }
}

// TODO move to oware
fn cleanup(cfg: Option<Res<OwareCfg>>) -> bool {
    cfg.map_or(false, |cfg| cfg.new_game || cfg.outcome.is_some())
}

#[derive(Resource, Clone, Copy)]
pub struct OwareCfg {
    pub human_is_first: bool,
    pub ai: Ai,
    pub new_game: bool,
    pub outcome: Option<Outcome>,
    pub init_seeds: u8,
}
impl Default for OwareCfg {
    fn default() -> Self {
        Self {
            human_is_first: true,
            ai: Ai::Random,
            outcome: None,
            new_game: false,
            init_seeds: if cfg!(feature = "dev") { 2 } else { 4 },
        }
    }
}
impl OwareCfg {
    pub fn get_actor(&self, player: Player) -> Actor {
        if player.index() != self.human_is_first as u8 {
            Actor::Human
        } else {
            Actor::Bot(self.ai)
        }
    }
    pub fn is_human(&self, player: Player) -> bool {
        player.index() != self.human_is_first as u8
    }
    pub fn outcome(&self) -> String {
        format!(
            "{:?}",
            self.outcome.pov(if self.human_is_first {
                Player::A
            } else {
                Player::B
            })
        )
    }
}

fn menu(mut commands: Commands, cfg: Option<Res<OwareCfg>>, state: Res<State<GameState>>) {
    let in_game = state.0 == GameState::Game;
    let sheet = Stylesheet::default()
        .with_background(BackgroundColor(Color::BLACK))
        .with_style(Style {
            position_type: if in_game {
                PositionType::Absolute
            } else {
                default()
            },
            ..default()
        });

    let new_game = cfg.is_none();
    if cfg.is_none() {
        commands.insert_resource(OwareCfg::default());
    }
    let cfg = cfg.map_or(OwareCfg::default(), |x| x.clone());
    commands.insert_resource(MenuState::new(
        cfg,
        if cfg.outcome.is_some() {
            Screens::GameOver
        } else if new_game {
            Screens::NewGame
        } else if in_game {
            Screens::Game
        } else {
            Screens::Pause
        },
        Some(sheet),
    ))
}
fn handle_events(
    mut action_event: EventReader<Actions>,
    #[cfg(not(target_arch = "wasm32"))] mut app_event: EventWriter<AppExit>,
    mut commands: Commands,
    menu_state: Option<Res<MenuState<Screens>>>,
) {
    if let Some(menu_state) = menu_state {
        if !action_event.is_empty() {
            commands.insert_resource(*menu_state.state());
        }
    }
    for event in action_event.iter() {
        match event {
            Actions::Resume | Actions::NewGame => {
                commands.insert_resource(NextState(Some(GameState::Game)))
            }
            Actions::Pause => commands.insert_resource(NextState(Some(GameState::Menu))),
            #[cfg(not(target_arch = "wasm32"))]
            Actions::Quit => app_event.send(AppExit),
            _ => (),
        }
    }
}

pub struct MenuPlugin;
impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(QuickMenuPlugin::<Screens>::new())
            .add_event::<Actions>()
            .add_startup_system(|mut commands: Commands| {
                commands.spawn(Camera2dBundle::default());
            })
            .add_system(menu.run_if(state_changed::<GameState>().and_then(not(in_state(GameState::Loading)))))
            .add_system(handle_events)
            .add_system(despawn_with::<PC>.run_if(cleanup.and_then(in_state(GameState::Menu))));
    }
}
