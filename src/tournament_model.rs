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
    async fn new(&self) -> Result<(), BotError>;
}

#[derive(Debug)]
pub struct SingleElimTournament {}

impl Tournament for SingleElimTournament {
    async fn new(&self) -> Result<(), BotError> {
        todo!();
    }
}
