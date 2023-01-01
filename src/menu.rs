use crate::loading::FontAssets;
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
        if state.inactives().is_empty() {
            Screens::Root
        } else if cfg.outcome.is_some() {
            Screens::GameOver
        } else {
            Screens::Resume
        },
        Some(sheet),
    ))
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Screens {
    Root,
    Resume,
    NewGame,
    GameOver,
}
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Actions {
    Close,
    Resume,
    Quit,
    NewGame,
    PlayerAsFirst,
    Bot(Ai),
}
pub enum MyEvent {
    CloseSettings,
    PopSettings,
    NewGame,
    Quit,
}
impl ActionTrait for Actions {
    type State = OwareCfg;
    type Event = MyEvent;
    fn handle(&self, state: &mut Self::State, event_writer: &mut EventWriter<Self::Event>) {
        match self {
            Actions::Close => event_writer.send(MyEvent::CloseSettings),
            Actions::PlayerAsFirst => state.human_is_first ^= true,
            Actions::Resume => event_writer.send(MyEvent::PopSettings),
            Actions::NewGame => event_writer.send(MyEvent::NewGame),
            Actions::Quit => event_writer.send(MyEvent::Quit),
            Actions::Bot(ai) => state.ai = *ai,
            // _ => unimplemented!(),
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
                Self::Root => {
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

fn exit_menu(
    mut my_event_er: EventReader<MyEvent>,
    mut exit_ew: EventWriter<AppExit>,
    mut state: ResMut<State<GameState>>,
    menu_stat: Res<MenuState<Screens>>,
    mut cfg: ResMut<OwareCfg>,
) {
    for event in my_event_er.iter() {
        *cfg = *menu_stat.state();
        match event {
            MyEvent::CloseSettings => state.set(GameState::Playing).unwrap(),
            MyEvent::PopSettings => state.pop().unwrap(),
            MyEvent::NewGame => {
                cfg.new_game = true;
                state.replace(GameState::Playing).unwrap()
            }
            MyEvent::Quit => exit_ew.send(AppExit),
        }
    }
}
fn cleanup(mut commands: Commands, cfg: Res<OwareCfg>, actors: Query<Entity, With<Actor>>) {
    bevy_quickmenu::cleanup(&mut commands);
    if cfg.new_game {
        actors.for_each(|e| commands.entity(e).despawn_recursive())
    }
}

#[derive(Component)]
pub struct Pause;
fn menu_button(mut commands: Commands, font_assets: Res<FontAssets>) {
    commands
        .spawn(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(120.0), Val::Px(50.0)),
                margin: UiRect::all(Val::Auto),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                ..Default::default()
            },
            background_color: Color::rgb(0., 0., 0.9).into(),
            ..Default::default()
        })
        .insert(Pause)
        .with_children(|parent| {
            parent.spawn(TextBundle {
                text: Text {
                    sections: vec![TextSection {
                        value: "Menu".to_string(),
                        style: TextStyle {
                            font: font_assets.fira_sans.clone(),
                            font_size: 40.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                        },
                    }],
                    alignment: Default::default(),
                },
                ..Default::default()
            });
        });
}
fn click_button(
    mut state: ResMut<State<GameState>>,
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Clicked => {
                state.push(GameState::Menu).unwrap();
            }
            Interaction::Hovered => {
                *color = Color::rgb(0.25, 0.25, 0.25).into();
            }
            Interaction::None => {
                *color = Color::rgb(0.15, 0.15, 0.15).into();
            }
        }
    }
}
fn despawn_children<T: Component>(mut commands: Commands, query: Query<Entity, With<T>>) {
    for item in query.iter() {
        commands.entity(item).despawn_recursive();
    }
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
            .add_event::<MyEvent>()
            .add_startup_system(setup_camera)
            .add_system_set(SystemSet::on_enter(GameState::Playing).with_system(menu_button))
            .add_system_set(SystemSet::on_resume(GameState::Playing).with_system(menu_button))
            .add_system_set(SystemSet::on_update(GameState::Playing).with_system(click_button))
            .add_system_set(
                SystemSet::on_enter(GameState::Menu).with_system(despawn_children::<Pause>),
            )
            .add_system_set(SystemSet::on_enter(GameState::Menu).with_system(setup_menu))
            .add_system_set(SystemSet::on_exit(GameState::Menu).with_system(cleanup))
            .add_system_set(SystemSet::on_update(GameState::Menu).with_system(exit_menu));
    }
}
