use crate::loading::{sprite, BoardAssets};
use crate::menu::OwareCfg;
use crate::tweens::*;
use crate::GameState;
use bevy::input::touch::TouchPhase;
use bevy::prelude::*;
use bevy::utils::HashMap;
use board_game::{
    board::{Board, Player},
    games::oware::OwareBoard,
};
use iyes_loopless::prelude::*;
use std::time::Duration;

mod components;
pub use components::*;
const SIZE: f32 = 50.;

#[cfg_attr(feature = "dev", derive(bevy_inspector_egui::Inspectable))]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Default)]
pub enum Ai {
    #[default]
    Random,
    Rollout(u32),
    Mcts(u32, u8),
    _MinMax,
}

pub fn entities_exist_with<T: Component>(query: Query<(), With<T>>) -> bool {
    !query.is_empty()
}

pub struct OwarePlugin<const P: usize>;

#[derive(Resource, Default, Deref, DerefMut)]
pub struct Oware<const P: usize>(OwareBoard<P>);

impl<const P: usize> Oware<P> {
    fn is_done(board: Res<Self>) -> bool {
        board.is_done()
    }
    fn _is_bot_turn(&self, cfg: Res<OwareCfg>) -> bool {
        self.next_player().index() == cfg.human_is_first as u8
    }
    pub fn seeds_in(&self, mv: &Bowl) -> u8 {
        if **mv < 2 * P {
            self.get_seeds(Player::BOTH[**mv / P], **mv % P)
        } else {
            self.score(Player::BOTH[**mv - 2 * P])
        }
    }
    pub fn is_player_bowl(&self, mv: &Bowl) -> bool {
        (if self.next_player().index() == 0 {
            0..P
        } else {
            P..2 * P
        })
        .contains(mv)
    }
}
impl<const P: usize> OwarePlugin<P> {
    fn spawn_board(
        mut commands: Commands,
        assets: Res<BoardAssets>,
        mut cfg: ResMut<OwareCfg>,
        mut board: ResMut<Oware<P>>,
        balls: Query<Entity, (With<Bowl>, Without<PC>)>,
    ) {
        if cfg.new_game {
            *board = Oware(OwareBoard::<P>::new(cfg.init_seeds));
            cfg.new_game = false;
            cfg.outcome = None;
            balls.for_each(|e| commands.entity(e).despawn_recursive())
        }
        let transform = |x, y| Transform::from_xyz(x, y, 1.);
        Player::BOTH.iter().enumerate().for_each(|(i, &player)| {
            let dir = if i == 0 { -1. } else { 1. };
            let v_off = (SIZE - 18.) * dir;
            commands
                .spawn(sprite(&assets.bowl, 100., transform(0., v_off * 4.)))
                .insert(PC(player))
                .insert(Bowl(2 * P + i))
                .insert(Interaction::None)
                .insert(Name::new(format!("Bowl{player:?}Score")))
                .with_children(|p| {
                    p.spawn(assets.text(
                        format!("{}", board.score(player)),
                        SIZE / 2.,
                        Color::BLACK,
                        transform(0., v_off * 2.),
                    ));
                });
            (0..P).for_each(|mv| {
                let h = dir * (SIZE * P as f32 / 2. - mv as f32 * SIZE - SIZE / 2.);
                let bowl = Bowl(mv + P * i);
                (0..board.init_seeds()).for_each(|i| {
                    commands
                        .spawn(sprite(&assets.meatball, SIZE / 4., transform(h, v_off)))
                        .insert(bowl.clone())
                        .insert(Name::new(format!("Seed{}", mv * 4 + i as usize)));
                });
                commands
                    .spawn(sprite(&assets.meatball_bowl, SIZE, transform(h, v_off)))
                    .insert(bowl)
                    .insert(PC(Player::BOTH[i]))
                    .insert(Interaction::None)
                    .insert(Name::new(format!("Bowl{player:?}{mv}")))
                    .with_children(|parent| {
                        parent.spawn(assets.text(
                            format!("{}", board.get_seeds(Player::A, mv),),
                            SIZE / 2.,
                            Color::BLACK,
                            transform(0., v_off),
                        ));
                        parent.spawn(assets.text(
                            format!("{}", mv + 1),
                            SIZE / 3.,
                            Color::rgba_u8(175, 163, 163, 255),
                            transform(0., -v_off / 2.),
                        ));
                    });
            })
        });
    }
    fn sow(
        mut commands: Commands,
        mut board: ResMut<Oware<P>>,
        bowls: Query<(&Bowl, &Transform, Option<&Moved>, Entity), With<PC>>,
        balls: Query<(Entity, &Bowl, Option<&MoveBall>), (Without<PC>, Without<Moved>)>,
    ) {
        let bowl_map: HashMap<usize, &Transform> = bowls.iter().map(|x| (**x.0, x.1)).collect();

        if let Some((sowed_bowl, entity)) = bowls.iter().find_map(|x| x.2.map(|_| (x.0, x.3))) {
            let mut next_bowl = sowed_bowl.next(sowed_bowl, 2 * P);
            balls
                .iter()
                .filter_map(|x| {
                    if x.1.eq(sowed_bowl) && x.2.is_none() {
                        Some(x.0)
                    } else {
                        None
                    }
                })
                .enumerate()
                .for_each(|(i, e)| {
                    commands.entity(e).insert(MoveBall(next_bowl.clone(), i));
                    next_bowl = next_bowl.next(sowed_bowl, 2 * P);
                });
            commands.entity(entity).remove::<Moved>();
            board.play(sowed_bowl.wrapping_rem(P));
        };

        let millis = |i: u64, f| Duration::from_millis(1 + i * f);
        balls.for_each(|(entity, from, movedball)| {
            if let Some(MoveBall(to_bowl, nth)) = movedball {
                let from = bowl_map.get(from).unwrap().translation;
                let to = bowl_map.get(to_bowl).unwrap().translation;
                let lerp = |a, b, r| a + (b - a) * r;
                let d = to.y - from.y;
                let p = SIZE / 2.;
                let b = 2. * (d.max(0.) + p + (p * p + p * d.abs()).sqrt());
                let tween = Delay::new(millis(*nth as u64, 243)).then(
                    BeTween::with_lerp(Duration::from_secs(2), move |tr: &mut Transform, _, r| {
                        tr.translation.x = lerp(from.x, to.x, r);
                        tr.translation.y = from.y - (b - d) * r.powi(2) + b * r;
                    })
                    .with_completed_event(3),
                );
                commands
                    .entity(entity)
                    .insert((Moved, Animator::new(tween)));
            }
        })
    }
    fn rm_ball(
        mut commands: Commands,
        moved: Query<&MoveBall>,
        mut completed: EventReader<TweenCompleted>,
    ) {
        for e in completed.iter() {
            let bowl = moved.get(e.entity).unwrap().0.clone();
            commands
                .entity(e.entity)
                .insert(bowl)
                .remove::<(Moved, MoveBall)>();
        }
    }
    fn update_scores(
        mut commands: Commands,
        board: Res<Oware<P>>,
        bowls: Query<&Bowl, With<PC>>,
        balls: Query<(Entity, &Bowl, Option<&MoveBall>), Without<PC>>,
    ) {
        if balls.iter().all(|x| x.2.is_none()) {
            bowls
                .iter()
                .filter(|x| board.seeds_in(x) == 0 && board.is_player_bowl(x))
                .for_each(|mv| {
                    balls
                        .iter()
                        .filter(|x| x.1.eq(mv))
                        .enumerate()
                        .for_each(|(i, x)| {
                            commands.entity(x.0).insert(MoveBall(
                                Bowl(board.next_player().other().index() as usize + 2 * P),
                                i,
                            ));
                        })
                })
        }
    }
    fn update_bowls(
        assets: Res<BoardAssets>,
        mut bowls: Query<(&Children, &PC, &Bowl, &mut Handle<Image>)>,
        cfg: Res<OwareCfg>,
        mut text: Query<&mut Text>,
        mut map: Local<HashMap<usize, bool>>,
        balls: Query<(Entity, &Bowl, Option<&MoveBall>), Without<PC>>,
    ) {
        let ball_count = balls
            .iter()
            .fold(HashMap::<usize, usize>::new(), |mut m, x| {
                let count = m.entry(**x.1).or_insert(0);
                *count += 1;
                m
            });
        bowls.for_each_mut(|(ch, player, mv, mut img, ..)| {
            let mut text = text.get_mut(ch[0]).unwrap();
            let &seeds = ball_count.get(mv).unwrap_or(&0);
            text.sections[0].value = format!(
                "{}{seeds}",
                if (2 * P).le(mv) {
                    format!("{:?}\n", cfg.get_actor(player.0))
                } else {
                    "".to_string()
                }
            );
            let balls = seeds > 1;
            if map.get(mv).map_or(true, |v| v ^ balls) {
                map.insert(**mv, balls);
                *img = if balls {
                    assets.meatball_bowl.clone()
                } else {
                    assets.bowl.clone()
                }
            }
        });
    }
    fn play(
        mut commands: Commands,
        board: Res<Oware<P>>,
        cfg: Res<OwareCfg>,
        bowls: Query<(Entity, &Interaction, &Bowl, &PC)>,
        time: Res<Time>,
        mut timer: Local<Timer>,
    ) {
        if board.next_player().index() == cfg.human_is_first as u8 {
            if timer.duration() == Duration::ZERO {
                *timer = Timer::new(Duration::from_millis(1729), TimerMode::Repeating);
            }
            if !timer.tick(time.delta()).just_finished() {
                return;
            }
        }
        let actor = cfg.get_actor(board.next_player());
        if let Some(mv) = if actor.is_human() {
            bowls
                .iter()
                .find(|e| e.1 == &Interaction::Clicked && cfg.is_human(e.3 .0))
                .map(|(_, _, Bowl(v), ..)| *v % P)
        } else {
            actor.get_mv(&board)
        } {
            if board.is_available_move(mv) {
                let bowl = bowls
                    .iter()
                    .find(|e| (mv + P * board.next_player().index() as usize).eq(e.2))
                    .unwrap()
                    .0;
                commands.entity(bowl).insert(Moved);
            }
        }
    }
    fn conclude_game(mut commands: Commands, mut cfg: ResMut<OwareCfg>, board: Res<Oware<P>>) {
        cfg.outcome = board.outcome();
        commands.insert_resource(NextState(GameState::Menu));
    }
    fn focus(
        mut pos: Local<Vec2>,
        mut cursor: EventReader<CursorMoved>,
        mut touch: EventReader<TouchInput>,
        mouse_button_inputs: Res<Input<MouseButton>>,
        kbd: Res<Input<KeyCode>>,
        mut actors: Query<(Entity, &GlobalTransform, Option<&Bowl>, &mut Interaction), With<PC>>,
    ) {
        if cursor.is_empty() && !kbd.is_changed() && !mouse_button_inputs.is_changed() {
            return;
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

        let k = kbd.get_just_released().find_map(|x| match x {
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
        });
        actors.for_each_mut(|mut e| {
            let on_bowl = Vec2::new(
                pos.x - e.1.translation().x - 300.,
                pos.y - e.1.translation().y - 400.,
            )
            .length()
                < SIZE / 2.;
            *e.3 = if k.map_or(
                on_bowl && mouse_button_inputs.just_released(MouseButton::Left),
                |k| e.2.map_or(false, |x| k == **x % P),
            ) {
                Interaction::Clicked
            } else if on_bowl {
                Interaction::Hovered
            } else {
                Interaction::None
            }
        });
    }
}

impl<const P: usize> Plugin for OwarePlugin<P> {
    fn build(&self, app: &mut App) {
        app.add_enter_system(GameState::Game, Self::spawn_board)
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::Game)
                    .with_system(Self::update_bowls)
                    .with_system(Self::sow)
                    .with_system(Self::focus.run_if_not(entities_exist_with::<Moved>))
                    .with_system(Self::update_scores.run_if_not(entities_exist_with::<Moved>))
                    .with_system(
                        Self::play
                            .run_if_not(Oware::<P>::is_done)
                            .run_if_not(entities_exist_with::<Moved>),
                    )
                    .with_system(Self::conclude_game.run_if(Oware::<P>::is_done))
                    .with_system(Self::rm_ball)
                    .into(),
            )
            .init_resource::<Oware<P>>();

        #[cfg(feature = "dev")]
        {
            use bevy_inspector_egui::RegisterInspectable;
            app.register_inspectable::<Bowl>();
        }
    }
}
