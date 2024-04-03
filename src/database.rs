use sqlx::PgPool;

use crate::models::{
    Config, ManagerRole, Match, MatchSchedule, Player, Tournament, TournamentPlayer,
};

/// Any database that the bot could use to operate the tournament
///
/// Note that changing the implementor of this trait will only allow you to change which database
/// you'll be using (e.g. Postgres, SQLite, etc.).
///
/// If you want to change the database schema, you'll need to change this trait as well as the models used by the implementor. 
#[allow(async_fn_in_trait)]
pub trait Database {
    type Error;
    // Represents records in each database table

    /// Tells you the manager for a given guild
    type ManagerRoleConfig;
    /// The various configurations for a given guild
    type GuildConfig;
    /// A tournament that may or may not be currently running
    type Tournament;
    /// A player (Discord user) that has registered with the bot
    type Player;
    /// Tells you which players are in which tournaments
    type TournamentPlayer;
    /// A match between two players in a tournament
    type Match;
    /// A proposed time for a match to take place and its status
    type MatchSchedule;

    /// Establishes a connection to the database and returns a handle to it
    async fn connect() -> Self;

    /// Creates all tables necessary for the tournament system
    ///
    /// This used in production to generate the tables at runtime.
    /// In development, use the build.rs script to generate the tables at compile time.
    async fn create_tables(&self) -> Result<(), Self::Error>;

    /// Sets the manager role for a guild
    async fn set_manager_role(
        &self,
        guild_id: String,
        manager_role_id: String,
    ) -> Result<(), Self::Error>;

    /// Sets the config for a guild
    async fn set_config(
        &self,
        guild_id: String,
        marshal_role_id: String,
        announcement_channel_id: String,
        notification_channel_id: String,
        log_channel_id: String,
    ) -> Result<(), Self::Error>;
}

/// The Postgres database used for the DBC tournament system
pub struct PgDatabase {
    pub pool: PgPool,
}

impl Database for PgDatabase {
    type Error = sqlx::Error;
    type ManagerRoleConfig = ManagerRole;
    type GuildConfig = Config;
    type Tournament = Tournament;
    type Player = Player;
    type TournamentPlayer = TournamentPlayer;
    type Match = Match;
    type MatchSchedule = MatchSchedule;

    async fn connect() -> Self {
        let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL was not set.");

        let pool = PgPool::connect(db_url.as_str())
            .await
            .expect("Failed to connect to the database.");

        PgDatabase { pool }
    }

    async fn create_tables(&self) -> Result<(), Self::Error> {
        sqlx::query_file!("migrations/20240330072934_create_config.sql")
            .execute(&self.pool)
            .await?;

        sqlx::query_file!("migrations/20240330072940_create_tournaments.sql")
            .execute(&self.pool)
            .await?;

        sqlx::query_file!("migrations/20240330072944_create_players.sql")
            .execute(&self.pool)
            .await?;

        sqlx::query_file!("migrations/20240330073147_create_tournament_players.sql")
            .execute(&self.pool)
            .await?;

        sqlx::query_file!("migrations/20240330073151_create_matches.sql")
            .execute(&self.pool)
            .await?;

        sqlx::query_file!("migrations/20240330073157_create_match_schedules.sql")
            .execute(&self.pool)
            .await?;

        sqlx::query_file!("migrations/20240330092559_create_manager_roles.sql")
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn set_manager_role(
        &self,
        guild_id: String,
        manager_role_id: String,
    ) -> Result<(), Self::Error> {
        sqlx::query!(
            r#"
            INSERT INTO manager_roles (guild_id, manager_role_id)
            VALUES ($1, $2)
            ON CONFLICT (guild_id)
            DO UPDATE SET
                manager_role_id = $2
            "#,
            guild_id,
            manager_role_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn set_config(
        &self,
        guild_id: String,
        marshal_role_id: String,
        announcement_channel_id: String,
        notification_channel_id: String,
        log_channel_id: String,
    ) -> Result<(), Self::Error> {
        sqlx::query!(
            r#"
            INSERT INTO config (guild_id, marshal_role_id, announcement_channel_id, notification_channel_id, log_channel_id)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (guild_id)
            DO UPDATE SET
                marshal_role_id = $2,
                announcement_channel_id = $3,
                notification_channel_id = $4,
                log_channel_id = $5
            "#,
            guild_id,
            marshal_role_id,
            announcement_channel_id,
            notification_channel_id,
            log_channel_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
