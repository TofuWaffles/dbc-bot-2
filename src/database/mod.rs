mod models;

use sqlx::PgPool;

use self::models::{GuildConfig, ManagerRoleConfig, Tournament, User};

/// Any database that the bot could use to operate the tournament
///
/// Note that changing the implementor of this trait will only allow you to change which database
/// you'll be using (e.g. Postgres, SQLite, etc.).
///
/// If you want to change the database schema, you'll need to change this trait as well as all its associated types
#[allow(async_fn_in_trait)]
pub trait Database {
    type Error;

    /// Establishes a connection to the database and returns a handle to it
    async fn connect() -> Result<Self, Self::Error>
    where
        Self: Sized;

    /// Creates all tables necessary for the tournament system
    ///
    /// This is used in production to generate the tables at runtime.
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

    async fn get_manager_role(
        &self,
        guild_id: &str,
    ) -> Result<Option<ManagerRoleConfig>, Self::Error>;

    async fn get_config(&self, guild_id: &str) -> Result<Option<GuildConfig>, Self::Error>;

    /// Adds a user to the database.
    async fn create_user(&self, discord_id: &str, player_tag: &str) -> Result<(), Self::Error>;

    /// Retrieves a user from the database.
    async fn get_user(&self, discord_id: &str) -> Result<Option<User>, Self::Error>;

    /// Creates a tournament in the database, returning the tournament id.
    async fn create_tournament(&self, guild_id: &str, name: &str) -> Result<i32, Self::Error>;

    /// Retrieves a tournament from the database.
    async fn get_tournament(
        &self,
        guild_id: &str,
        tournament_id: &i32,
    ) -> Result<Option<Tournament>, Self::Error>;

    /// Retrieves all tournaments from the database.
    async fn get_all_tournaments(&self, guild_id: &str) -> Result<Vec<Tournament>, Self::Error>;

    /// Retrieves all active tournaments from the database.
    async fn get_active_tournaments(&self, guild_id: &str) -> Result<Vec<Tournament>, Self::Error>;
}

/// The Postgres database used for the DBC tournament system
pub struct PgDatabase {
    pub pool: PgPool,
}

impl Database for PgDatabase {
    type Error = sqlx::Error;

    async fn connect() -> Result<Self, Self::Error> {
        let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL was not set.");

        let pool = PgPool::connect(db_url.as_str()).await?;

        Ok(PgDatabase { pool })
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

    async fn get_manager_role(
        &self,
        guild_id: &str,
    ) -> Result<Option<ManagerRoleConfig>, Self::Error> {
        let manager_role = sqlx::query_as!(
            ManagerRoleConfig,
            r#"
            SELECT * FROM manager_roles WHERE guild_id = $1
            LIMIT 1
            "#,
            guild_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(manager_role)
    }

    async fn get_config(&self, guild_id: &str) -> Result<Option<GuildConfig>, Self::Error> {
        let config = sqlx::query_as!(
            GuildConfig,
            r#"
            SELECT * FROM config WHERE guild_id = $1
            LIMIT 1
            "#,
            guild_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(config)
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

    async fn get_user(&self, discord_id: &str) -> Result<Option<User>, Self::Error> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT * FROM users WHERE discord_id = $1
            LIMIT 1
            "#,
            discord_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    async fn create_tournament(&self, guild_id: &str, name: &str) -> Result<i32, Self::Error> {
        let now = chrono::offset::Utc::now();

        let tournament_id = sqlx::query!(
            r#"
            INSERT INTO tournaments (guild_id, name, created_at, active, started)
            VALUES ($1, $2, $3, true, false)
            RETURNING tournament_id
            "#,
            guild_id,
            name,
            now
        )
        .fetch_one(&self.pool)
        .await?
        .tournament_id;

        Ok(tournament_id)
    }

    async fn get_tournament(
        &self,
        guild_id: &str,
        tournament_id: &i32,
    ) -> Result<Option<Tournament>, Self::Error> {
        let tournament = sqlx::query_as!(
            Tournament,
            r#"
            SELECT * FROM tournaments WHERE guild_id = $1 AND tournament_id = $2
            ORDER BY created_at DESC
            "#,
            guild_id,
            tournament_id,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(tournament)
    }

    async fn get_all_tournaments(&self, guild_id: &str) -> Result<Vec<Tournament>, Self::Error> {
        let tournaments = sqlx::query_as!(
            Tournament,
            r#"
            SELECT * FROM tournaments WHERE guild_id = $1
            ORDER BY created_at DESC
            "#,
            guild_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(tournaments)
    }

    async fn get_active_tournaments(&self, guild_id: &str) -> Result<Vec<Tournament>, Self::Error> {
        let tournaments = sqlx::query_as!(
            Tournament,
            r#"
            SELECT * FROM tournaments WHERE guild_id = $1 AND active = true
            "#,
            guild_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(tournaments)
    }
}
