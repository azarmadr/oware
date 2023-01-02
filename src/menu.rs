use crate::oware::{Actor, Ai};
use crate::GameState;
use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_quickmenu::style::Stylesheet;
use bevy_quickmenu::{ActionTrait, Menu, MenuItem, MenuState, QuickMenuPlugin, ScreenTrait};
use board_game::board::{Outcome, Player};
use board_game::pov::NonPov;

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
fn setup_menu(mut commands: Commands, state: Res<State<GameState>>, cfg: Res<OwareCfg>) {
    let sheet = Stylesheet::default().with_background(BackgroundColor(Color::BLACK));

    commands.insert_resource(MenuState::new(
        *cfg,
        if state.inactives().is_empty() && state.current() == &GameState::Menu {
            Screens::Init
        } else if cfg.outcome.is_some() {
            Screens::GameOver
        } else if state.inactives().is_empty() {
            Screens::Pause
        } else {
            Screens::Resume
        },
        Some(sheet),
    ))
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Screens {
    Init,
    Pause,
    Resume,
    NewGame,
    GameOver,
}
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Actions {
    Close,
    Resume,
    Menu,
    Quit,
    NewGame,
    PlayerAsFirst,
    Bot(Ai),
}
impl ActionTrait for Actions {
    type State = OwareCfg;
    type Event = Self;
    fn handle(&self, state: &mut Self::State, event_writer: &mut EventWriter<Self::Event>) {
        match self {
            Self::Close | Self::Menu | Self::Resume | Self::NewGame | Self::Quit => {
                event_writer.send(*self)
            }
            Self::PlayerAsFirst => state.human_is_first ^= true,
            Self::Bot(ai) => state.ai = *ai,
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
        let bot_list = &mut [
            Ai::RandomBot,
            Ai::RolloutBot(27),
            Ai::RolloutBot(729),
            Ai::MCTSBot(27, 1),
            Ai::MCTSBot(729, 2),
        ]
        .iter()
        .map(|&x| MenuItem::action(format!("{x:?}"), Actions::Bot(x)).checked(state.ai == x))
        .collect::<Vec<MenuItem<Screens>>>();
        Menu::new(
            format!("{self:?}"),
            match self {
                Self::Resume => vec![
                    MenuItem::headline("Paused"),
                    MenuItem::action("Resume", Actions::Resume),
                    MenuItem::screen("New Game", Screens::NewGame),
                    MenuItem::action("Quit", Actions::Quit),
                ],
                Self::Pause => vec![MenuItem::action("Pause", Actions::Menu)],
                Self::GameOver => vec![
                    MenuItem::headline(state.outcome()),
                    MenuItem::screen("New Game", Screens::NewGame),
                    MenuItem::action("Quit", Actions::Quit),
                ],
                Self::NewGame => {
                    let mut items = vec![
                        MenuItem::headline("New Game"),
                        MenuItem::action("Start", Actions::NewGame),
                        MenuItem::label("Player Position"),
                        MenuItem::action("Is First", Actions::PlayerAsFirst)
                            .checked(state.human_is_first),
                        MenuItem::label("Bot Type"),
                    ];
                    items.append(bot_list);
                    items
                }
                Self::Init => {
                    let mut items = vec![
                        MenuItem::headline("Oware"),
                        MenuItem::action("Start the Game", Actions::Close),
                        MenuItem::label("Configuration"),
                        MenuItem::label("Player Position"),
                        MenuItem::action("Is First", Actions::PlayerAsFirst)
                            .checked(state.human_is_first),
                        MenuItem::label("Bot Type"),
                    ];
                    items.append(bot_list);
                    items
                }
            },
        )
    }
}

fn show_menu(mut my_event_er: EventReader<Actions>, mut state: ResMut<State<GameState>>) {
    for event in my_event_er.iter() {
        match event {
            Actions::Menu => state.push(GameState::Menu).unwrap(),
            _ => (),
        }
    }
}
fn handle_events(
    mut my_event_er: EventReader<Actions>,
    mut exit_ew: EventWriter<AppExit>,
    mut state: ResMut<State<GameState>>,
    mut cfg: ResMut<OwareCfg>,
    menu_state: Res<MenuState<Screens>>,
) {
    for event in my_event_er.iter() {
        *cfg = *menu_state.state();
        match event {
            Actions::Close => state.set(GameState::Playing).unwrap(),
            Actions::Resume => state.pop().unwrap(),
            Actions::NewGame => {
                cfg.new_game = true;
                state.replace(GameState::Playing).unwrap()
            }
            Actions::Quit => exit_ew.send(AppExit),
            _ => (),
        }
    }
}

// TODO move to oware
fn cleanup(mut commands: Commands, cfg: Res<OwareCfg>, actors: Query<Entity, With<Actor>>) {
    if cfg.new_game {
        actors.for_each(|e| commands.entity(e).despawn_recursive())
    }
}

#[derive(Component)]
pub struct Pause;
fn menu_button(mut commands: Commands) {
    let sheet = Stylesheet::default()
        .with_background(BackgroundColor(Color::BLACK))
        .with_style(Style {
            position_type: PositionType::Absolute,
            ..default()
        });

    commands.insert_resource(MenuState::new(
        OwareCfg::default(),
        Screens::Pause,
        Some(sheet),
    ));
}
#[derive(Resource, Clone, Copy)]
pub struct OwareCfg {
    pub human_is_first: bool,
    pub ai: Ai,
    pub new_game: bool,
    pub outcome: Option<Outcome>,
}
impl Default for OwareCfg {
    fn default() -> Self {
        Self {
            human_is_first: true,
            ai: Ai::RandomBot,
            outcome: None,
            new_game: false,
        }
    }
}
impl OwareCfg {
    pub fn get_actor(&self, idx: usize) -> Actor {
        if idx != self.human_is_first as usize {
            Actor::Human
        } else {
            Actor::Bot(self.ai)
        }
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

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(QuickMenuPlugin::<Screens>::new())
            .insert_resource(OwareCfg::default())
            .add_event::<Actions>()
            .add_startup_system(setup_camera)
            .add_system_set(SystemSet::on_enter(GameState::Playing).with_system(menu_button))
            .add_system_set(SystemSet::on_resume(GameState::Playing).with_system(menu_button))
            .add_system_set(SystemSet::on_update(GameState::Playing).with_system(show_menu))
            .add_system_set(SystemSet::on_enter(GameState::Menu).with_system(setup_menu))
            .add_system_set(SystemSet::on_exit(GameState::Menu).with_system(cleanup))
            .add_system_set(SystemSet::on_update(GameState::Menu).with_system(handle_events));
    }
}
