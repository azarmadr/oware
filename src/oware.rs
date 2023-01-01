use crate::loading::BoardAssets;
use crate::menu::OwareCfg;
use crate::GameState;
use bevy::input::touch::TouchPhase;
use bevy::prelude::*;
use board_game::ai::mcts::MCTSBot;
use board_game::ai::simple::{RandomBot, RolloutBot};
use board_game::ai::Bot;
use board_game::board::Board;
use board_game::{board::Player, games::oware::OwareBoard};
// use menu_plugin::MenuMaterials;

#[derive(Component)]
pub struct Bowl(usize);

#[derive(Component, Clone, Copy, Debug)]
pub enum Actor {
    Bot(Ai),
    Human,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Ai {
    RandomBot,
    RolloutBot(u32),
    MCTSBot(u32, u8),
    _MinMaxBot,
}

fn play_with_bot<const P: usize>(board: &mut OwareBoard<P>, mut bot: Box<dyn Bot<OwareBoard<P>>>) {
    let mv = bot.select_move(board);
    board.play(mv)
}

impl Actor {
    pub fn is_human(&self) -> bool {
        match self {
            Self::Bot(_) => false,
            _ => true,
        }
    }
    pub fn play<const P: usize>(&self, board: &mut OwareBoard<P>, mv: Option<usize>) {
        match self {
            Self::Human => {
                if let Some(mv) = mv {
                    if board.is_available_move(mv) {
                        board.play(mv)
                    }
                }
            }
            Self::Bot(ai) => {
                let rng = rand::thread_rng();
                play_with_bot(
                    board,
                    match ai {
                        Ai::RandomBot => Box::new(RandomBot::new(rng)),
                        Ai::RolloutBot(r) => Box::new(RolloutBot::new(*r, rng)),
                        Ai::MCTSBot(i, ew) => Box::new(MCTSBot::new(*i as u64, *ew as f32, rng)),
                        _ => unimplemented!("Ai moves"),
                    },
                )
            }
        }
    }
}

pub struct OwarePlugin<const P: usize>;

// #[cfg_attr(feature = "dev", derive(bevy_inspector_egui::Inspectable))]
#[derive(Resource, Default, Deref, DerefMut)]
pub struct Oware<const P: usize>(OwareBoard<P>);

/// This plugin handles player related stuff like movement
/// Player logic is only active during the State `GameState::Playing`
impl<const P: usize> Plugin for OwarePlugin<P> {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(GameState::Playing).with_system(Self::spawn_board))
            .add_system_set(SystemSet::on_update(GameState::Playing)
                            .with_system(Self::update_bowls)
                            .with_system(Self::play)
                            .with_system(Self::get_mv)
                            )
            .init_resource::<Oware<P>>()
            // .init_resource::<MenuMaterials>()
            ;
    }
}

impl<const P: usize> OwarePlugin<P> {
    fn spawn_board(
        mut commands: Commands,
        assets: Res<BoardAssets>,
        mut cfg: ResMut<OwareCfg>,
        mut board: ResMut<Oware<P>>,
    ) {
        if cfg.new_game {
            *board = Oware::<P>::default();
            cfg.new_game = false;
        }
        [Player::A, Player::B]
            .iter()
            .enumerate()
            .for_each(|(i, &player)| {
                let dir = if i == 0 { -1. } else { 1. };
                let v_off = 22. * dir;
                let actor = cfg.get_actor(i);
                commands
                    .spawn(SpriteBundle {
                        texture: assets.bowl.clone(),
                        transform: Transform::from_translation(Vec3::new(0., 4. * v_off, 1.))
                            .with_scale(Vec3::new(2.7, 2.7, 0.)),
                        ..default()
                    })
                    .insert(actor)
                    .insert(Name::new(format!("Bowl{player:?}Score")))
                    .with_children(|p| {
                        p.spawn(Text2dBundle {
                            text: Text::from_section(
                                format!("{}", board.score(player)),
                                TextStyle {
                                    font: assets.fira_sans.clone(),
                                    font_size: 18.,
                                    color: Color::BLACK,
                                    ..default()
                                },
                            )
                            .with_alignment(TextAlignment::CENTER),
                            transform: Transform::from_translation(Vec3::new(0., v_off, 2.)),
                            ..default()
                        });
                    });
                (0..P).for_each(|mv| {
                    let h = dir * (30. * P as f32 / 2. - mv as f32 * 30. - 15.);
                    commands
                        .spawn(SpriteBundle {
                            texture: assets.bowl.clone(),
                            transform: Transform::from_translation(Vec3::new(h, v_off, 1.)),
                            ..default()
                        })
                        .insert(Bowl(mv))
                        .insert(actor)
                        .insert(Name::new(format!("Bowl{player:?}{mv}")))
                        .with_children(|p| {
                            p.spawn(Text2dBundle {
                                text: Text::from_section(
                                    format!("{}", board.get_seeds(Player::A, mv)),
                                    TextStyle {
                                        font: assets.fira_sans.clone(),
                                        font_size: 20.,
                                        color: Color::BLACK,
                                        ..default()
                                    },
                                )
                                .with_alignment(TextAlignment::CENTER),
                                transform: Transform::from_translation(Vec3::new(0., v_off, 2.)),
                                ..default()
                            });
                            p.spawn(Text2dBundle {
                                text: Text::from_section(
                                    format!("{}", mv + 1),
                                    TextStyle {
                                        font: assets.fira_sans.clone(),
                                        font_size: 13.,
                                        color: Color::rgba_u8(175, 163, 163, 255),
                                        ..default()
                                    },
                                )
                                .with_alignment(TextAlignment::CENTER),
                                transform: Transform::from_translation(Vec3::new(
                                    0.,
                                    -v_off / 2.,
                                    2.,
                                )),
                                ..default()
                            });
                        });
                })
            });
    }
    fn update_bowls(
        board: Res<Oware<P>>,
        bowls: Query<(&Children, &Name, &Actor, Option<&Bowl>)>,
        mut text: Query<&mut Text>,
    ) {
        bowls.for_each(|(ch, n, actor, mv)| {
            let player = if n.contains("lA") {
                Player::A
            } else {
                Player::B
            };
            let mut text = text.get_mut(ch[0]).unwrap();
            text.sections[0].value = format!(
                "{}",
                mv.map_or(format!("{actor:?}\n{}", board.score(player)), |mv| board
                    .get_seeds(player, mv.0)
                    .to_string())
            );
        });
    }
    fn get_mv() {}
    fn play(
        mut commands: Commands,
        mut state: ResMut<State<GameState>>,
        mut board: ResMut<Oware<P>>,
        actors: Query<(Entity, &GlobalTransform, &Bowl, &Name, &Actor)>,
        mv: Res<Input<KeyCode>>,
        mut cfg: ResMut<OwareCfg>,
        mouse_button_inputs: Res<Input<MouseButton>>,
        mut cursor: EventReader<CursorMoved>,
        mut touch: EventReader<TouchInput>,
        mut pos: Local<Vec2>,
    ) {
        *pos = cursor.iter().last().map_or(
            touch.iter().last().map_or(*pos, |x| {
                if x.phase == TouchPhase::Ended {
                    x.position
                } else {
                    *pos
                }
            }),
            |x| x.position,
        );
        let mv = (if mv.just_released(KeyCode::Key1) {
            Some(0)
        } else if mv.just_released(KeyCode::Key2) {
            Some(1)
        } else if mv.just_released(KeyCode::Key3) {
            Some(2)
        } else if mv.just_released(KeyCode::Key4) {
            Some(3)
        } else if mv.just_released(KeyCode::Key5) {
            Some(4)
        } else if mv.just_released(KeyCode::Key6) {
            Some(5)
        } else if mv.just_released(KeyCode::Key7) {
            Some(6)
        } else if mv.just_released(KeyCode::Key8) {
            Some(7)
        } else if mv.just_released(KeyCode::Key9) {
            Some(8)
        } else if mouse_button_inputs.just_released(MouseButton::Left) {
            bevy::log::info!("{pos:?}");
            actors
                .iter()
                .find(|e| {
                    e.4.is_human()
                        && Vec2::new(
                            pos.x - e.1.translation().x - 300.,
                            pos.y - e.1.translation().y - 400.,
                        )
                        .length()
                            < 16.
                })
                .map(|(_, _, Bowl(v), ..)| *v)
        } else {
            None
        })
        .map_or(None, |x| if x < P { Some(x) } else { None });
        if !board.is_done() {
            actors
                .iter()
                .find(|&x| x.3.contains(&format!("l{}", board.next_player().to_char())))
                .unwrap()
                .4
                .play(&mut board, mv);
        } else {
            cfg.outcome = board.outcome();
            state.push(GameState::Menu).unwrap();
            actors.for_each(|e| commands.entity(e.0).despawn_recursive())
        }
    }
}
