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
use iyes_loopless::prelude::*;
use std::time::Duration;

#[derive(Component)]
pub struct Moved;

#[cfg_attr(feature = "dev", derive(bevy_inspector_egui::Inspectable))]
#[derive(Component, Debug)]
pub struct Bowl(usize);
impl Bowl {
    pub fn mv(&self) -> usize {
        match self {
            Bowl(x) => *x,
        }
    }
    pub fn is(&self, value: usize) -> bool {
        self.mv() == value
    }
}

#[cfg_attr(feature = "dev", derive(bevy_inspector_egui::Inspectable))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Actor {
    Bot(Ai),
    Human,
}
#[derive(Component, Deref, Debug)]
pub struct PC(Player);

#[cfg_attr(feature = "dev", derive(bevy_inspector_egui::Inspectable))]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Default)]
pub enum Ai {
    #[default]
    Random,
    Rollout(u32),
    Mcts(u32, u8),
    _MinMax,
}

impl Actor {
    pub fn is_human(&self) -> bool {
        !matches!(self, Self::Bot(_))
    }
    pub fn get_mv<const P: usize>(&self, board: &OwareBoard<P>) -> Option<usize> {
        match self {
            Self::Human => None,
            Self::Bot(ai) => {
                let rng = rand::thread_rng();
                Some(match ai {
                    Ai::Random => RandomBot::new(rng).select_move(board),
                    Ai::Rollout(r) => RolloutBot::new(*r, rng).select_move(board),
                    Ai::Mcts(i, ew) => MCTSBot::new(*i as u64, *ew as f32, rng).select_move(board),
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

impl<const P: usize> Oware<P> {
    fn is_done(board: Res<Self>) -> bool {
        board.is_done()
    }
    fn _is_bot_turn(&self, cfg: Res<OwareCfg>) -> bool {
        self.next_player().index() == cfg.human_is_first as u8
    }
}
pub fn moving(moves: Query<(), With<Moved>>) -> bool {
    !moves.is_empty()
}
const SIZE: f32 = 50.;
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
        let transform = |x, y| Transform::from_xyz(x, y, 1.);
        Player::BOTH.iter().enumerate().for_each(|(i, &player)| {
            let dir = if i == 0 { -1. } else { 1. };
            let v_off = (SIZE - 18.) * dir;
            commands
                .spawn(assets.bowl_sprite(100., transform(0., v_off * 4.)))
                .insert(PC(Player::BOTH[i]))
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
                commands
                    .spawn(assets.meatball_bowl_sprite(50., transform(h, v_off)))
                    .insert(Bowl(mv + P * i))
                    .insert(PC(Player::BOTH[i]))
                    .insert(Interaction::None)
                    .insert(Name::new(format!("Bowl{player:?}{mv}")))
                    .with_children(|p| {
                        p.spawn(assets.text(
                            format!("{}", board.get_seeds(Player::A, mv),),
                            SIZE / 2.,
                            Color::BLACK,
                            transform(0., v_off),
                        ));
                        p.spawn(assets.text(
                            format!("{}", mv + 1),
                            SIZE / 3.,
                            Color::rgba_u8(175, 163, 163, 255),
                            transform(0., -v_off / 2.),
                        ));
                    });
            })
        });
    }
    fn rm_ball(
        mut commands: Commands,
        // moved: Query<(Entity, &Animator<Transform>), With<Moved>>,
        mut completed: EventReader<TweenCompleted>,
    ) {
        for e in completed.iter() {
            commands.entity(e.entity).despawn_recursive();
        }
    }
    fn update_bowls(
        mut commands: Commands,
        mut board: ResMut<Oware<P>>,
        assets: Res<BoardAssets>,
        mut bowls: Query<(
            &Children,
            &PC,
            Option<&Bowl>,
            &mut Handle<Image>,
            &Transform,
        )>,
        cfg: Res<OwareCfg>,
        mut text: Query<&mut Text>,
        mut map: Local<HashMap<String, bool>>,
        moved: Query<(Entity, &Bowl, &PC), With<Moved>>,
    ) {
        if moved.is_empty() {
            return;
        }

        let (moved, mv, PC(player)) = moved.get_single().unwrap();
        let mv = mv.mv();
        let modulo_dist = |x| (x + 2 * P - mv) % (2 * P);
        let mut seeds = board.get_seeds(*player, mv % P);
        let (mut fro, mut to): (Vec<_>, Vec<_>) = bowls
            .iter()
            .filter_map(|(c, _, b, _, t)| b.map(|b| (b, t, c)))
            .filter(|(x, ..)| {
                seeds as usize >= 2 * P as usize || modulo_dist(x.mv()) <= seeds as usize
            })
            .partition(|x| x.0.mv() == mv);
        let fro = fro.pop().unwrap();
        to.sort_by_key(|x| modulo_dist(x.0.mv()));

        let mut iter = to.iter().cycle().enumerate();

        let millis = |i: u64, f| Duration::from_millis(1 + i * f);
        commands
            .entity(fro.2[0])
            .insert(Animator::new(BeTween::with_lerp(
                millis(seeds as u64, 243),
                move |t: &mut Text, _, r| {
                    t.sections[0].value = format!("{}", (seeds as f32 * (1. - r)).floor())
                },
            )));
        while seeds > 0 {
            let (i, next) = iter.next().unwrap();
            let f = fro.1.translation;
            let t = next.1.translation;
            let lerp = |a, b, r| a + (b - a) * r;
            let d = t.y - f.y;
            let p = SIZE / 2.;
            let b = 2. * (d.max(0.) + p + (p * p + p * d.abs()).sqrt());
            let tween = Delay::new(millis(i as u64, 243)).then(
                BeTween::with_lerp(Duration::from_secs(2), move |tr: &mut Transform, _, r| {
                    tr.translation.x = lerp(f.x, t.x, r);
                    tr.translation.y = f.y - (b - d) * r.powi(2) + b * r;
                })
                .with_completed_event(3),
            );
            // seeds = 0;
            seeds -= 1;
            commands
                .spawn((Animator::new(tween), Moved))
                .insert(assets.meatball_sprite(SIZE / 4., *fro.1));
        }

        board.play(mv % P);

        commands.entity(moved).remove::<Moved>();
        bowls.for_each_mut(|(ch, player, mv, mut img, ..)| {
            let actor = cfg.get_actor(player.0);
            let mut text = text.get_mut(ch[0]).unwrap();
            let hash = format!("{mv:?}{player:?}");
            let seeds = mv.map_or(board.score(player.0), |mv| {
                board.get_seeds(player.0, mv.0 % P)
            });
            text.sections[0].value = format!(
                "{}{seeds}",
                if mv.is_none() {
                    format!("{actor:?}\n")
                } else {
                    "".to_string()
                }
            );
            if map.get(&hash).map_or(true, |v| v ^ (seeds > 0)) {
                map.insert(hash, seeds > 0);
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
        board: Res<Oware<P>>,
        cfg: Res<OwareCfg>,
        actors: Query<(Entity, &Interaction, &Bowl, &PC)>,
        time: Res<Time>,
        mut timer: Local<Timer>,
    ) {
        if board.next_player().index() == cfg.human_is_first as u8 {
            if timer.duration() == Duration::ZERO {
                timer.set_duration(Duration::from_millis(1729));
                timer.set_mode(TimerMode::Repeating)
            }
            if !timer.tick(time.delta()).just_finished() {
                return;
            }
        }
        let actor = cfg.get_actor(board.next_player());
        if let Some(mv) = if actor.is_human() {
            actors
                .iter()
                .find(|e| e.1 == &Interaction::Clicked && cfg.is_human(e.3 .0))
                .map(|(_, _, Bowl(v), ..)| *v % P)
        } else {
            actor.get_mv(&board)
        } {
            if board.is_available_move(mv) {
                let bowl = actors
                    .iter()
                    .find(|e| e.2.is(mv + P * board.next_player().index() as usize))
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
        // board: Res<>
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
                |k| e.2.map_or(false, |x| k == x.mv() % P),
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
                    .with_system(Self::focus.run_if_not(moving))
                    .with_system(
                        Self::play
                            .run_if_not(Oware::<P>::is_done)
                            .run_if_not(moving),
                    )
                    .with_system(Self::conclude_game.run_if(Oware::<P>::is_done))
                    .with_system(Self::rm_ball)
                    .into(),
            )
            .init_resource::<Oware<P>>();
    }
}
