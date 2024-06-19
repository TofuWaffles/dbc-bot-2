use chrono::DateTime;
use sqlx::PgPool;

use crate::BotError;

use self::models::{
    GuildConfig, ManagerRoleConfig, Match, MatchSchedule, PlayerType, Tournament, TournamentStatus,
    User,
};

/// Models for the database
///
/// These models are specific to the current database design and schema.
/// Most if not all are directly mapped to a table in the database.
pub mod models;

/// Any database that the bot could use to operate the tournament.
///
/// Note that changing the implementor of this trait will only allow you to change which database
/// you'll be using (e.g. Postgres, SQLite, etc.).
///
/// If you want to change the database schema, you'll need to change this trait as well as all its associated types.
#[allow(async_fn_in_trait)]
pub trait Database {
    type Error;

    /// Establishes a connection to the database and returns a handle to it.
    async fn connect() -> Result<Self, Self::Error>
    where
        Self: Sized;

    /// Sets the manager role for a guild.
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

    /// Retrieves the manager role from the database.
    async fn get_manager_role(
        &self,
        guild_id: &str,
    ) -> Result<Option<ManagerRoleConfig>, Self::Error>;

    /// Retrieves the config of a given guild from the database.
    async fn get_config(&self, guild_id: &str) -> Result<Option<GuildConfig>, Self::Error>;

    /// Adds a user to the database.
    async fn create_user(&self, discord_id: &str, player_tag: &str) -> Result<(), Self::Error>;

    /// Retrieves a user from the database.
    async fn get_user(&self, discord_id: &str) -> Result<Option<User>, Self::Error>;

    /// Creates a tournament in the database, returning the tournament id.
    async fn create_tournament(
        &self,
        guild_id: &str,
        name: &str,
        tournament_id: Option<&i32>,
    ) -> Result<i32, Self::Error>;

    /// Updates the status of a tournament.
    async fn set_tournament_status(
        &self,
        tournament_id: &i32,
        new_status: TournamentStatus,
    ) -> Result<(), Self::Error>;

    /// Retrieves a tournament from the database given a guild id and tournament id.
    async fn get_tournament(
        &self,
        guild_id: &str,
        tournament_id: &i32,
    ) -> Result<Option<Tournament>, Self::Error>;

    /// Retrieves all tournaments from the database.
    async fn get_all_tournaments(&self, guild_id: &str) -> Result<Vec<Tournament>, Self::Error>;

    /// Retrieves all active tournaments from the database.
    async fn get_active_tournaments(&self, guild_id: &str) -> Result<Vec<Tournament>, Self::Error>;

    /// Retrieves all active tournaments that the player has currently entered.
    ///
    /// Note: in the current design, a player can only be in one active tournament at a time.
    /// This rule should be enforced at the bot command level.
    /// This method will still return multiple active tournaments if the player is in multiple active tournaments.
    async fn get_player_active_tournaments(
        &self,
        guild_id: &str,
        discord_id: &str,
    ) -> Result<Vec<Tournament>, Self::Error>;

    /// Deletes a tournament from the database.
    async fn delete_tournament(&self, tournament_id: &i32) -> Result<(), Self::Error>;

    /// Enters a user into a tournament.
    async fn enter_tournament(
        &self,
        tournament_id: &i32,
        discord_id: &str,
    ) -> Result<(), Self::Error>;

    /// Gets all players in a tournament.
    async fn get_tournament_players(&self, tournament_id: &i32) -> Result<Vec<User>, Self::Error>;

    /// Creates a match associated with a tournament.
    async fn create_match(
        &self,
        tournament_id: &i32,
        round: &i32,
        sequence_in_round: &i32,
        player_1_type: PlayerType,
        player_2_type: PlayerType,
        discord_id_1: Option<&str>,
        discord_id: Option<&str>,
    ) -> Result<(), Self::Error>;

    /// Retrieves a match by its id.
    async fn get_match_by_id(&self, match_id: &str) -> Result<Option<Match>, Self::Error>;

    /// Retrieves a match by the player's discord id.
    ///
    /// This will retrive the match with the highest round number that does not yet have a winner.
    async fn get_match_by_player(
        &self,
        tournament_id: &i32,
        discord_id: &str,
    ) -> Result<Option<Match>, Self::Error>;

    /// Retrieves all matches associated with a tournament.
    ///
    /// Pass in a None for the round number to retrieve all matches for the tournament.
    async fn get_matches_by_tournament(
        &self,
        tournament_id: &i32,
        round: Option<&i32>,
    ) -> Result<Vec<Match>, Self::Error>;

    /// Create a new match schedule in the database.
    async fn create_or_update_match_schedule(
        &self,
        match_id: &str,
        proposed_time: u64,
        time_of_proposal: u64,
        proposer: i32,
    ) -> Result<(), Self::Error>;

    /// Retrieves a match schedule from the database.
    async fn get_match_schedule(
        &self,
        match_id: &str,
    ) -> Result<Option<MatchSchedule>, Self::Error>;

    /// Set the accetped value of a match value.
    async fn set_match_schedule_accepted(
        &self,
        match_id: &str,
        accepted: bool,
    ) -> Result<(), Self::Error>;
}

/// The Postgres database used for the DBC tournament system.
#[derive(Debug)]
pub struct PgDatabase {
    pool: PgPool,
}

impl Database for PgDatabase {
    type Error = BotError;

    async fn connect() -> Result<Self, Self::Error> {
        #[cfg(debug_assertions)]
        dotenv::dotenv().ok();

        let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL was not set.");

        let pool = PgPool::connect(db_url.as_str()).await?;

        Ok(PgDatabase { pool })
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

    async fn create_tournament(
        &self,
        guild_id: &str,
        name: &str,
        tournament_id: Option<&i32>,
    ) -> Result<i32, Self::Error> {
        let timestamp_time = chrono::offset::Utc::now().timestamp();

        let tournament_id = match tournament_id {
            None => {
                sqlx::query!(
                    r#"
            INSERT INTO tournaments (guild_id, name, created_at)
            VALUES ($1, $2, $3)
            RETURNING tournament_id
            "#,
                    guild_id,
                    name,
                    timestamp_time
                )
                .fetch_one(&self.pool)
                .await?
                .tournament_id
            }
            Some(custom_id) => {
                sqlx::query!(
                    r#"
            INSERT INTO tournaments (guild_id, name, created_at, tournament_id)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (tournament_id) DO NOTHING
            "#,
                    guild_id,
                    name,
                    timestamp_time,
                    custom_id
                )
                .execute(&self.pool)
                .await?;

                *custom_id
            }
        };

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
            SELECT tournament_id, guild_id, name, status as "status: _", created_at, start_time
            FROM tournaments WHERE guild_id = $1 AND tournament_id = $2
            ORDER BY created_at DESC
            LIMIT 1
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
            SELECT tournament_id, guild_id, name, status as "status: _", created_at, start_time
            FROM tournaments WHERE guild_id = $1
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
            SELECT tournament_id, guild_id, name, status as "status: _", created_at, start_time
            FROM tournaments WHERE guild_id = $1 AND (status = 'pending' OR status = 'started')
            "#,
            guild_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(tournaments)
    }

    async fn enter_tournament(
        &self,
        tournament_id: &i32,
        discord_id: &str,
    ) -> Result<(), Self::Error> {
        sqlx::query!(
            r#"
            INSERT INTO tournament_players (tournament_id, discord_id)
            VALUES ($1, $2)
            ON CONFLICT (tournament_id, discord_id)
            DO NOTHING
            "#,
            tournament_id,
            discord_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_tournament_players(&self, tournament_id: &i32) -> Result<Vec<User>, Self::Error> {
        let players = sqlx::query_as!(
            User,
            r#"
            SELECT users.discord_id, users.player_tag
            FROM tournament_players
            JOIN users ON tournament_players.discord_id = users.discord_id
            WHERE tournament_players.tournament_id = $1
            "#,
            tournament_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(players)
    }

    async fn create_match(
        &self,
        tournament_id: &i32,
        round: &i32,
        sequence_in_round: &i32,
        player_1_type: PlayerType,
        player_2_type: PlayerType,
        discord_id_1: Option<&str>,
        discord_id_2: Option<&str>,
    ) -> Result<(), Self::Error> {
        let match_id = Match::generate_id(tournament_id, round, sequence_in_round);

        sqlx::query!(
            r#"
            INSERT INTO matches (match_id, tournament_id, round, sequence_in_round, player_1_type, player_2_type, discord_id_1, discord_id_2)
            VALUES ($1, $2, $3, $4, $5::player_type, $6::player_type, $7, $8)
            ON CONFLICT (match_id) DO NOTHING
            "#,
            match_id,
            tournament_id,
            round,
            sequence_in_round,
            player_1_type as PlayerType,
            player_2_type as PlayerType,
            discord_id_1,
            discord_id_2
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_match_by_id(&self, match_id: &str) -> Result<Option<Match>, Self::Error> {
        let bracket = sqlx::query_as!(
            Match,
            r#"
            SELECT match_id, tournament_id, round, sequence_in_round, player_1_type as "player_1_type: _", player_2_type as "player_2_type: _", discord_id_1, discord_id_2, winner
            FROM matches
            WHERE match_id = $1
            ORDER BY round DESC
            LIMIT 1
            "#,
            match_id
            )
            .fetch_optional(&self.pool)
            .await?;

        Ok(bracket)
    }

    async fn get_match_by_player(
        &self,
        tournament_id: &i32,
        discord_id: &str,
    ) -> Result<Option<Match>, Self::Error> {
        let bracket = sqlx::query_as!(
            Match,
            r#"
            SELECT match_id, tournament_id, round, sequence_in_round, player_1_type as "player_1_type: _", player_2_type as "player_2_type: _", discord_id_1, discord_id_2, winner
            FROM matches
            WHERE tournament_id = $1 AND (discord_id_1 = $2 OR discord_id_2 = $2) AND winner IS NULL
            ORDER BY round DESC
            LIMIT 1
            "#,
            tournament_id,
            discord_id,
            )
            .fetch_optional(&self.pool)
            .await?;

        Ok(bracket)
    }

    async fn get_matches_by_tournament(
        &self,
        tournament_id: &i32,
        round: Option<&i32>,
    ) -> Result<Vec<Match>, Self::Error> {
        let brackets = match round {
            Some(round) => sqlx::query_as!(
                Match,
                r#"
                SELECT match_id, tournament_id, round, sequence_in_round, player_1_type as "player_1_type: _", player_2_type as "player_2_type: _", discord_id_1, discord_id_2, winner
                FROM matches
                WHERE tournament_id = $1 AND round = $2
                ORDER BY sequence_in_round
                "#,
                tournament_id,
                round
                )
                .fetch_all(&self.pool)
                .await?,
            None => sqlx::query_as!(
                Match,
                r#"
                SELECT match_id, tournament_id, round, sequence_in_round, player_1_type as "player_1_type: _", player_2_type as "player_2_type: _", discord_id_1, discord_id_2, winner
                FROM matches
                WHERE tournament_id = $1
                ORDER BY round DESC, sequence_in_round
                "#,
                tournament_id
                )
                .fetch_all(&self.pool)
                .await?,
        };

        Ok(brackets)
    }

    async fn delete_tournament(&self, tournament_id: &i32) -> Result<(), Self::Error> {
        sqlx::query!(
            r#"
            DELETE FROM tournaments
            WHERE tournament_id = $1
            "#,
            tournament_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_player_active_tournaments(
        &self,
        guild_id: &str,
        discord_id: &str,
    ) -> Result<Vec<Tournament>, Self::Error> {
        let tournaments = sqlx::query_as!(
            Tournament,
            r#"
            SELECT tournaments.tournament_id, tournaments.guild_id, tournaments.name, tournaments.status as "status: _", tournaments.created_at, tournaments.start_time
            FROM tournaments
            INNER JOIN tournament_players ON tournaments.tournament_id=tournament_players.tournament_id
            WHERE tournaments.guild_id = $1 AND (tournaments.status = 'pending' OR tournaments.status = 'started') AND tournament_players.discord_id = $2
            "#,
            guild_id,
            discord_id,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(tournaments)
    }

    async fn set_tournament_status(
        &self,
        tournament_id: &i32,
        new_status: TournamentStatus,
    ) -> Result<(), Self::Error> {
        sqlx::query!(
            r#"
            UPDATE tournaments
            SET status = $2
            WHERE tournament_id = $1
            "#,
            tournament_id,
            new_status as TournamentStatus,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn create_or_update_match_schedule(
        &self,
        match_id: &str,
        proposed_time: u64,
        time_of_proposal: u64,
        proposer: i32,
    ) -> Result<(), Self::Error> {
        sqlx::query!(
            r#"
            INSERT INTO match_schedules (match_id, proposed_time, time_of_proposal, proposer, accepted)
            VALUES($1, $2, $3, $4, false)
            ON CONFLICT (match_id) DO UPDATE
                SET match_id = $1,
                    proposed_time = $2,
                    time_of_proposal = $3,
                    proposer = $4;
            "#,
            match_id,
            proposed_time as i64,
            time_of_proposal as i64,
            proposer as i16,
            )
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn get_match_schedule(
        &self,
        match_id: &str,
    ) -> Result<Option<MatchSchedule>, Self::Error> {
        let match_schedule_model = sqlx::query!(
            r#"
            SELECT match_id, proposed_time, time_of_proposal, proposer, accepted
            FROM match_schedules
            WHERE match_id = $1;
            "#,
            match_id
        )
        .fetch_optional(&self.pool)
        .await?;

        match match_schedule_model {
            Some(match_schedule) => Ok(Some(MatchSchedule {
                match_id: match_schedule.match_id,
                proposed_time: DateTime::from_timestamp(match_schedule.proposed_time, 0).unwrap(),
                time_of_proposal: DateTime::from_timestamp(match_schedule.time_of_proposal, 0)
                    .unwrap(),
                proposer: match_schedule.proposer,
                accepted: match_schedule.accepted,
            })),
            None => Ok(None),
        }
    }

    async fn set_match_schedule_accepted(
        &self,
        match_id: &str,
        accepted: bool,
    ) -> Result<(), Self::Error> {
        sqlx::query!(
            r#"
            UPDATE match_schedules
            SET accepted = $2
            WHERE match_id = $1;
            "#,
            match_id,
            accepted
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
