use crate::BotError;

/// Defines a model for managing tournaments.
///
/// Implementors of this trait should be used to create and manage tournaments.
///
/// By default, the tournament style is Single Elimination, but implementors can be changed to
/// manaage other types of tournaments and even connect to third-party APIs, if they so wish.
///
/// The caveat is that tournament model is tightly coupled the commands that control it, so more
/// work needs to be done to change the tournament style than just simply changing the implementor.
#[allow(async_fn_in_trait)]
pub trait TournamentModel {
    async fn create_tournament(&self) -> Result<(), BotError>;
}

#[derive(Debug)]
pub struct SingleElimTournament {}

impl TournamentModel for SingleElimTournament {
    async fn create_tournament(&self) -> Result<(), BotError> {
        todo!();
    }
}
