use crate::loading::BoardAssets;
use crate::menu::OwareCfg;
use crate::tweens::*;
use crate::GameState;
use bevy::input::touch::TouchPhase;
use bevy::prelude::*;
use bevy::utils::HashMap;
use board_game::ai::mcts::MCTSBot;
use board_game::ai::simple::{RandomBot, RolloutBot};
use board_game::ai::Bot;
use board_game::board::Board;
use board_game::{board::Player, games::oware::OwareBoard};
use std::time::Duration;

#[derive(Component)]
pub struct Moved;

#[derive(Component)]
pub struct Bowl(usize);

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
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

impl Actor {
    pub fn is_human(&self) -> bool {
        match self {
            Self::Bot(_) => false,
            _ => true,
        }
    }
    pub fn get_mv<const P: usize>(&self, board: &OwareBoard<P>) -> Option<usize> {
        match self {
            Self::Human => None,
            Self::Bot(ai) => {
                let rng = rand::thread_rng();
                Some(match ai {
                    Ai::RandomBot => RandomBot::new(rng).select_move(board),
                    Ai::RolloutBot(r) => RolloutBot::new(*r, rng).select_move(board),
                    Ai::MCTSBot(i, ew) => {
                        MCTSBot::new(*i as u64, *ew as f32, rng).select_move(board)
                    }
                    _ => unimplemented!("Ai moves"),
                })
            }
        }
    }
}

pub struct OwarePlugin<const P: usize>;

// #[cfg_attr(feature = "dev", derive(bevy_inspector_egui::Inspectable))]
#[derive(Resource, Default, Deref, DerefMut)]
pub struct Oware<const P: usize>(OwareBoard<P>);

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
                            texture: assets.meatball_bowl.clone(),
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
        mut commands: Commands,
        board: Res<Oware<P>>,
        assets: Res<BoardAssets>,
        mut bowls: Query<(&Children, &Name, &Actor, Option<&Bowl>, &mut Handle<Image>)>,
        mut text: Query<&mut Text>,
        mut map: Local<HashMap<Name, bool>>,
        moved: Query<Entity, With<Moved>>,
    ) {
        moved.for_each(|e| {
            let tween = Tween::new(
                EaseFunction::ElasticInOut,
                Duration::from_secs(2),
                BeTween::with_lerp(|t: &mut Transform, s, r| {
                    t.translation = s
                        .translation
                        .lerp(s.translation + Vec3::new(0., 33., 3.), r)
                }),
            )
            .with_repeat_count(RepeatCount::Finite(2))
            .with_repeat_strategy(RepeatStrategy::MirroredRepeat);
            commands
                .entity(e)
                .insert(Animator::new(tween))
                .remove::<Moved>();
        });
        bowls.for_each_mut(|(ch, n, actor, mv, mut img)| {
            let player = if n.contains("lA") {
                Player::A
            } else {
                Player::B
            };
            let mut text = text.get_mut(ch[0]).unwrap();
            let seeds = mv.map_or(board.score(player), |mv| board.get_seeds(player, mv.0));
            text.sections[0].value = format!(
                "{}{seeds}",
                if mv.is_none() {
                    format!("{actor:?}\n")
                } else {
                    "".to_string()
                }
            );
            if map.get(n).map_or(true, |v| v ^ (seeds > 0)) {
                map.insert(n.clone(), seeds > 0);
                *img = if seeds > 0 {
                    assets.meatball_bowl.clone()
                } else {
                    assets.bowl.clone()
                }
            }
        });
    }
    fn play(
        mut commands: Commands,
        mut state: ResMut<State<GameState>>,
        mut board: ResMut<Oware<P>>,
        mut cfg: ResMut<OwareCfg>,
        actors: Query<(Entity, &GlobalTransform, &Bowl, &Name, &Actor)>,
        time: Res<Time>,
        mut timer: Local<Timer>,
        mut pos: Local<Vec2>,
        mut cursor: EventReader<CursorMoved>,
        mut touch: EventReader<TouchInput>,
        mouse_button_inputs: Res<Input<MouseButton>>,
        mv: Res<Input<KeyCode>>,
    ) {
        if (board.next_player().index() < 1) ^ cfg.human_is_first {
            if timer.duration() == Duration::ZERO {
                timer.set_duration(Duration::from_millis(1729));
                timer.set_mode(TimerMode::Repeating)
            }
            if !timer.tick(time.delta()).just_finished() {
                return;
            }
        }
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
        let actor = actors
            .iter()
            .find(|&x| x.3.contains(&format!("l{}", board.next_player().to_char())))
            .unwrap()
            .4;
        let mv = if actor.is_human() {
            mv.get_just_released()
                .filter_map(|x| match x {
                    KeyCode::Key1 => Some(0),
                    KeyCode::Key2 => Some(1),
                    KeyCode::Key3 => Some(2),
                    KeyCode::Key4 => Some(3),
                    KeyCode::Key5 => Some(4),
                    KeyCode::Key6 => Some(5),
                    KeyCode::Key7 => Some(6),
                    KeyCode::Key8 => Some(7),
                    KeyCode::Key9 => Some(8),
                    _ => None,
                })
                .find(|&x| x < P)
                .or(actors
                    .iter()
                    .find(|e| {
                        mouse_button_inputs.just_released(MouseButton::Left)
                            && e.4.is_human()
                            && Vec2::new(
                                pos.x - e.1.translation().x - 300.,
                                pos.y - e.1.translation().y - 400.,
                            )
                            .length()
                                < 16.
                    })
                    .map(|(_, _, Bowl(v), ..)| *v))
        } else {
            actor.get_mv(&board)
        };
        if !board.is_done() {
            if mv.map_or(false, |mv| board.is_available_move(mv)) {
                board.play(mv.unwrap());
                let bowl = actors
                    .iter()
                    .find(|(_, _, Bowl(x), _, &a)| a == *actor && *x == mv.unwrap())
                    .unwrap()
                    .0;
                commands.entity(bowl).insert(Moved);
            }
        } else {
            cfg.outcome = board.outcome();
            state.push(GameState::Menu).unwrap();
            actors.for_each(|e| commands.entity(e.0).despawn_recursive())
        }
    }
}

impl<const P: usize> Plugin for OwarePlugin<P> {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(GameState::Playing).with_system(Self::spawn_board))
            .add_system_set(SystemSet::on_update(GameState::Playing)
                            .with_system(Self::update_bowls)
                            .with_system(Self::play)
                            // .with_system(Self::get_mv)
                            )
            .init_resource::<Oware<P>>()
            // .init_resource::<MenuMaterials>()
            ;
    }
}
