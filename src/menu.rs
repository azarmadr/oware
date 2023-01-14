use crate::oware::{Actor, Ai, PC};
use crate::{despawn_with, GameState};
use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_quickmenu::{style::Stylesheet, *};
use board_game::board::{Outcome, Player};
use board_game::pov::NonPov;
use iyes_loopless::prelude::*;

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
fn sm(mut commands: Commands) {
    let sheet = Stylesheet::default().with_background(BackgroundColor(Color::BLACK));

    commands.insert_resource(OwareCfg::default());
    commands.insert_resource(MenuState::new(
        OwareCfg::default(),
        Screens::NewGame,
        Some(sheet),
    ))
}
fn setup_menu(mut commands: Commands, cfg: Res<OwareCfg>) {
    let sheet = Stylesheet::default().with_background(BackgroundColor(Color::BLACK));

    commands.insert_resource(MenuState::new(
        *cfg,
        if cfg.outcome.is_some() {
            Screens::GameOver
        } else {
            Screens::Resume
        },
        Some(sheet),
    ))
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Screens {
    Pause,
    Resume,
    NewGame,
    GameOver,
}
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Actions {
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
            Self::Menu | Self::Resume | Self::NewGame | Self::Quit => event_writer.send(*self),
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
                        MenuItem::headline("Oware"),
                        MenuItem::action("Start a New Game", Actions::NewGame),
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

fn show_menu(mut my_event_er: EventReader<Actions>, mut commands: Commands) {
    for event in my_event_er.iter() {
        if event == &Actions::Menu {
            commands.insert_resource(NextState(GameState::Menu))
        }
    }
}
fn handle_events(
    mut my_event_er: EventReader<Actions>,
    mut exit_ew: EventWriter<AppExit>,
    mut commands: Commands,
    mut cfg: ResMut<OwareCfg>,
    menu_state: Res<MenuState<Screens>>,
) {
    for event in my_event_er.iter() {
        *cfg = *menu_state.state();
        match event {
            Actions::Resume | Actions::NewGame => {
                commands.insert_resource(NextState(GameState::Game))
            }
            Actions::Quit => exit_ew.send(AppExit),
            _ => (),
        }
        cfg.new_game = Actions::NewGame == *event;
    }
}

// TODO move to oware
fn cleanup(cfg: Res<OwareCfg>) -> bool {
    cfg.new_game || cfg.outcome.is_some()
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

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(QuickMenuPlugin::<Screens>::new())
            // .insert_resource(OwareCfg::default())
            .add_event::<Actions>()
            .add_startup_system(setup_camera)
            .add_enter_system(GameState::Game, menu_button)
            .add_exit_system(GameState::Game, setup_menu)
            .add_exit_system(GameState::Loading, sm)
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::Menu)
                    .with_system(handle_events)
                    .with_system(despawn_with::<PC>.run_if(cleanup))
                    .into(),
            )
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::Game)
                    .with_system(show_menu)
                    .into(),
            );
    }
}
