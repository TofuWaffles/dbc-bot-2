use sqlx::PgPool;

use crate::models::{
    Config, ManagerRole, Match, MatchSchedule, User, Tournament, TournamentPlayer,
};

/// Any database that the bot could use to operate the tournament
///
/// Note that changing the implementor of this trait will only allow you to change which database
/// you'll be using (e.g. Postgres, SQLite, etc.).
///
/// If you want to change the database schema, you'll need to change this trait as well as all its associated types
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
    type User;
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
        guild_id: &str,
        manager_role_id: &str,
    ) -> Result<(), Self::Error>;

    /// Sets the config for a guild
    async fn set_config(
        &self,
        guild_id: &str,
        marshal_role_id: &str,
        announcement_channel_id: &str,
        notification_channel_id: &str,
        log_channel_id: &str,
    ) -> Result<(), Self::Error>;

    async fn create_user(&self, discord_id: &str, player_tag: &str) -> Result<(), Self::Error>;

    async fn get_user(&self, discord_id: &str) -> Result<Option<Self::User>, Self::Error>;
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
    type User = User;
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
        // This isn't the most reliable way to create the tables
        // I might change this to be inlined in the future
        sqlx::query_file!("migrations/20240330072934_create_config.sql")
            .execute(&self.pool)
            .await?;

        sqlx::query_file!("migrations/20240330072940_create_tournaments.sql")
            .execute(&self.pool)
            .await?;

        sqlx::query_file!("migrations/20240330072936_create_users.sql")
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
        guild_id: &str,
        manager_role_id: &str,
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
        guild_id: &str,
        marshal_role_id: &str,
        announcement_channel_id: &str,
        notification_channel_id: &str,
        log_channel_id: &str,
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

    async fn get_user(&self, discord_id: &str) -> Result<Option<Self::User>, Self::Error> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT * FROM users WHERE discord_id = $1
            "#,
            discord_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    async fn create_user(&self, discord_id: &str, player_tag: &str) -> Result<(), Self::Error> {
        sqlx::query!(
            r#"
            INSERT INTO users (discord_id, player_tag)
            VALUES ($1, $2)
            ON CONFLICT (discord_id)
            DO UPDATE SET
                player_tag = $2
            "#,
            discord_id,
            player_tag
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
