use std::ops::Deref;

use bevy::prelude::*;
use board_game::{
    ai::mcts::MCTSBot,
    ai::simple::{RandomBot, RolloutBot},
    ai::Bot,
    board::Player,
    games::oware::OwareBoard,
};

use super::Ai;

#[derive(Component)]
pub struct Moved;

#[derive(Component, Debug, Clone)]
pub struct MoveBall(pub Bowl, pub usize);

// #[cfg_attr(feature = "dev", derive(bevy_inspector_egui::Inspectable))]
#[derive(Component, Debug, Clone, Deref, DerefMut, PartialEq, Eq)]
pub struct Bowl(pub usize);
impl Bowl {
    pub fn next(&self, skip: &Self, wrap: usize) -> Self {
        let next_move = Self((self.deref() + 1) % wrap);
        if next_move.eq(skip) {
            next_move.next(skip, wrap)
        } else {
            next_move
        }
    }
}

// #[cfg_attr(feature = "dev", derive(bevy_inspector_egui::Inspectable))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Actor {
    Bot(Ai),
    Human,
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

#[derive(Component, Deref, Debug)]
pub struct PC(pub Player);
