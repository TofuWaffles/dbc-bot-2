use crate::{database::Database, BotError};

#[allow(async_fn_in_trait)]
pub trait TournamentModel<DB>
    where DB: crate::database::Database
{
    async fn create_tournament(&self, db: &DB) -> Result<(), BotError>;
}

pub struct SingleElimTournament<DB> {
    pub database: DB,
}

impl<DB> TournamentModel<DB> for SingleElimTournament<DB>
    where DB: Database
{
    async fn create_tournament(&self, db: &DB) -> Result<(), BotError> {
        db.create_tables().await?;
        Ok(())
    }
}
