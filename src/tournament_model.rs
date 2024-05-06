use std::{collections::HashMap, sync::{Arc, Mutex}};

use crate::BotError;

/// Defines a model for managing tournaments.
///
/// implementations should be used to create and manage tournaments.
///
/// By default, the tournament style is Single Elimination, but implementations can be changed to
/// manaage other types of tournaments and even connect to third-party APIs, if they so wish.
///
/// The caveat is that tournament model is tightly coupled the commands that control it, so more
/// work needs to be done to change the tournament style than just simply changing the implementations.
#[allow(async_fn_in_trait)]
pub trait Tournament {
}

#[derive(Debug)]
pub struct SingleElimTournament {
    final_bracket: Arc<Mutex<Bracket>>,
    active_brackets: Arc<Mutex<HashMap<String, Bracket>>>,
}

impl SingleElimTournament {
    pub fn new() -> Self {
        todo!();
    }
}

impl Tournament for SingleElimTournament {
}

#[derive(Debug)]
pub struct Bracket {
    player_1: String,
    player_2: String,
    winner: String,
}
