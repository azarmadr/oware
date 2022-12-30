use crate::loading::BoardAssets;
use crate::GameState;
use bevy::prelude::*;
use board_game::board::Board;
use board_game::{board::Player, games::oware::OwareBoard};
// use menu_plugin::MenuMaterials;

#[derive(Component)]
pub struct Bowl(usize);

#[derive(Component)]
pub enum Actor {
    Bot(Player),
    Human(Player),
}

impl Actor {
    pub fn player(&self) -> Player {
        match self {
            Self::Bot(pl) => *pl,
            Self::Human(pl) => *pl,
        }
    }
    pub fn is(&self, player: Player) -> bool {
        self.player() == player
    }
    pub fn play<const P: usize>(&self, board: &mut OwareBoard<P>, mv: Option<usize>) {
        match self {
            Self::Human(_) => {
                if let Some(mv) = mv {
                    if board.is_available_move(mv) {
                        board.play(mv)
                    }
                }
            }
            Self::Bot(_) => {
                let mv = board.random_available_move(&mut rand::thread_rng());
                board.play(mv)
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
                            )
            .init_resource::<Oware<P>>()
            // .init_resource::<MenuMaterials>()
            ;
    }
}

impl<const P: usize> OwarePlugin<P> {
    fn spawn_board(mut commands: Commands, assets: Res<BoardAssets>, board: Res<Oware<P>>) {
        [Player::A, Player::B]
            .iter()
            .enumerate()
            .for_each(|(i, &player)| {
                let dir = if i == 0 { -1. } else { 1. };
                let v_off = 22. * dir;
                let actor = if i == 0 {
                    Actor::Human(player)
                } else {
                    Actor::Bot(player)
                };
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
                                    font_size: 20.,
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
                    let actor = if i == 0 {
                        Actor::Human(player)
                    } else {
                        Actor::Bot(player)
                    };
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
        // mut commands: Commands,
        // assets: Res<BoardAssets>,
        board: Res<Oware<P>>,
        bowls: Query<(&Children, &Actor, Option<&Bowl>)>,
        mut text: Query<&mut Text>,
    ) {
        bowls.for_each(|(ch, a, mv)| {
            let mut text = text.get_mut(ch[0]).unwrap();
            text.sections[0].value = format!(
                "{}",
                mv.map_or(board.score(a.player()), |mv| board
                    .get_seeds(a.player(), mv.0))
            );
        });
    }
    fn play(mut board: ResMut<Oware<P>>, actors: Query<&Actor>, mv: Res<Input<KeyCode>>) {
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
        } else {
            None
        })
        .map_or(None, |x| if x < P { Some(x) } else { None });
        if !board.is_done() {
            actors
                .iter()
                .find(|x| x.is(board.next_player()))
                .unwrap()
                .play(&mut board, mv);
        }
    }
}
