use crate::{database::Database, BotError};

#[allow(async_fn_in_trait)]
pub trait TournamentModel {
    async fn create_tournament(&self) -> Result<(), BotError>;
}

pub struct SingleElimTournament {}

impl TournamentModel for SingleElimTournament {
    async fn create_tournament(&self) -> Result<(), BotError> {
        todo!();
    }
}
