use sqlx::PgPool;

/// Any database that the bot could use to operate the tournament
#[allow(async_fn_in_trait)]
pub trait Database {

    /// Establishes a connection to the database and returns a handle to it
    async fn new() -> Self;

    /// Creates all tables necessary for the tournament system
    ///
    /// This used in production to generate the tables at runtime.
    /// In development, use the build.rs script to generate the tables at compile time.
    async fn create_tables(&self) -> Result<(), sqlx::Error>;
    async fn set_config(
        &self,
        guild_id: String,
        manager_role_id: String,
        host_role_id: String,
        announcement_channel_id: String,
        notification_channel_id: String,
        log_channel_id: String,
    ) -> Result<(), sqlx::Error>;
}

/// The Postgres database used for the DBC tournament system
pub struct PgDatabase {
    pool: PgPool,
}

impl Database for PgDatabase {
    async fn new() -> Self {
        let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL was not set.");

        let pool = PgPool::connect(db_url.as_str())
            .await
            .expect("Failed to connect to the database.");

        PgDatabase { pool }
    }

    async fn create_tables(&self) -> Result<(), sqlx::Error> {
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

        Ok(())
    }

    async fn set_config(
        &self,
        guild_id: String,
        manager_role_id: String,
        host_role_id: String,
        announcement_channel_id: String,
        notification_channel_id: String,
        log_channel_id: String,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO config (guild_id, manager_role_id, host_role_id, announcement_channel_id, notification_channel_id, log_channel_id)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (guild_id)
            DO UPDATE SET
                manager_role_id = $2,
                host_role_id = $3,
                announcement_channel_id = $4,
                notification_channel_id = $5,
                log_channel_id = $6
            "#,
            guild_id,
            manager_role_id,
            host_role_id,
            announcement_channel_id,
            notification_channel_id,
            log_channel_id
        )
        .execute(&self.pool)
        .await?;


        Ok(())
    }
}
