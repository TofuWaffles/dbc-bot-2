use crate::info;
use crate::BotError;
use models::Battle;
use models::BattleClass;
use models::BattleResult;
use models::BattleType;
use models::Event;
use models::Mode;
use models::Player;
use models::Tournament;
use sqlx::PgPool;

use self::models::{
    GuildConfig, ManagerRoleConfig, Match, PlayerNumber, PlayerType, TournamentStatus, User,
};

/// Models for the database.
///
/// These models are specific to the current database design and schema.
/// Most if not all are directly mapped to a table in the database.
pub mod models;

/// Any database that the bot could use to operate the tournament.
///
/// Note that changing the implementation of this trait will only allow you to change which database
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

    async fn migrate(&self) -> Result<(), Self::Error>;

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
        log_channel_id: &str,
        announcement_channel_id: &str,
    ) -> Result<(), Self::Error>;

    /// Retrieves the manager role from the database.
    async fn get_manager_role(
        &self,
        guild_id: &str,
    ) -> Result<Option<ManagerRoleConfig>, Self::Error>;

    /// Retrieves the config of a given guild from the database.
    async fn get_config(&self, guild_id: &str) -> Result<Option<GuildConfig>, Self::Error>;

    /// Adds a user to the database.
    async fn create_user(&self, user: &User) -> Result<(), Self::Error>;

    /// Deletes a user from the database.
    async fn delete_user(&self, discord_id: &str) -> Result<(), Self::Error>;

    /// Retrieves a user from the database with a given Discord ID.
    async fn get_player_by_discord_id(
        &self,
        discord_id: &str,
    ) -> Result<Option<Player>, Self::Error>;

    /// Retrieves a user from the database with a given player tag.
    async fn get_player_by_player_tag(
        &self,
        player_tag: &str,
    ) -> Result<Option<Player>, Self::Error>;

    /// Retrieves a user from the database with a given player.
    async fn get_user_by_player(&self, player: Player) -> Result<Option<User>, Self::Error>;

    /// Creates a tournament in the database, returning the tournament id.
    async fn create_tournament(
        &self,
        guild_id: &str,
        name: &str,
        tournament_id: impl Into<Option<i32>>,
        role_id: String,
        announcement_channel_id: &str,
        notification_channel_id: &str,
        wins_required: i32,
    ) -> Result<i32, Self::Error>;

    /// Updates the status of a tournament.
    async fn set_tournament_status(
        &self,
        tournament_id: i32,
        new_status: TournamentStatus,
    ) -> Result<(), Self::Error>;

    /// Retrieves a tournament from the database given a guild id and tournament id.
    async fn get_tournament(
        &self,
        guild_id: &str,
        tournament_id: i32,
    ) -> Result<Option<Tournament>, Self::Error>;

    /// Retrieves all tournaments from the database.
    async fn get_all_tournaments(&self, guild_id: &str) -> Result<Vec<Tournament>, Self::Error>;

    /// Retrieves all active tournaments from the database.
    ///
    /// This will get all active tournaments that have their status set to either "pending",
    /// "started", or "paused".
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
    async fn delete_tournament(&self, tournament_id: i32) -> Result<(), Self::Error>;

    /// Sets the current map for a given tournament.
    ///
    /// All matches must be done in the current map in order for them to be counted.
    async fn set_map(&self, tournament_id: i32, map: &str) -> Result<(), Self::Error>;

    /// Enters a user into a tournament.
    async fn enter_tournament(
        &self,
        tournament_id: i32,
        discord_id: &str,
    ) -> Result<(), Self::Error>;

    /// Exits a user from a tournament.
    async fn exit_tournament(
        &self,
        tournament_id: &i32,
        discord_id: &str,
    ) -> Result<(), Self::Error>;

    /// Get an active tournament of a player by their discord id.
    async fn get_active_tournaments_from_player(
        &self,
        discord_id: &str,
    ) -> Result<Vec<Tournament>, Self::Error>;

    /// Sets the number of wins required to win each round of the tournament.
    async fn set_wins_required(
        &self,
        tournament_id: &i32,
        wins_required: &i32,
    ) -> Result<(), Self::Error>;

    /// Gets all players in a tournament.
    async fn get_tournament_players(&self, tournament_id: i32) -> Result<Vec<Player>, Self::Error>;

    /// Updates the total number of rounds a tournament has.
    ///
    /// Useful for when a tournament starts because the number of rounds can only be determined
    /// when the number of contestants are known.
    async fn set_rounds(&self, tournament_id: i32, rounds: i32) -> Result<(), Self::Error>;

    /// Increments the current round of a tournament by 1.
    ///
    /// The caller is responsible to check if calls to this method will make a tournament's current
    /// round exceed its total number of rounds.
    async fn next_round(&self, tournament_id: i32) -> Result<(), Self::Error>;

    /// Creates a match associated with a tournament.
    async fn create_match(
        &self,
        tournament_id: i32,
        round: i32,
        sequence_in_round: i32,
        player_1_type: PlayerType,
        player_2_type: PlayerType,
        discord_id_1: Option<&str>,
        discord_id_2: Option<&str>,
    ) -> Result<(), Self::Error>;

    async fn add_records(&self, match_id: &str) -> Result<i64, Self::Error>;

    async fn add_battle(&self, battle: Battle) -> Result<i64, Self::Error>;

    async fn add_battle_class(&self, battle_class: BattleClass) -> Result<i64, Self::Error>;

    async fn add_event(&self, event: Event) -> Result<i64, Self::Error>;

    /// Sets the ready status of a player of a specified match to true.
    async fn set_ready(
        &self,
        match_id: &str,
        player_number: &PlayerNumber,
    ) -> Result<(), Self::Error>;

    /// Sets the winner of a match
    async fn set_winner(
        &self,
        match_id: &str,
        player_number: PlayerNumber,
    ) -> Result<(), Self::Error>;

    /// Retrieves a match by its id.
    async fn get_match_by_id(&self, match_id: &str) -> Result<Option<Match>, Self::Error>;

    /// Retrieves a match by the player's discord id.
    ///
    /// This will retrive the match with the highest round number that does not yet have a winner.
    async fn get_match_by_player(
        &self,
        tournament_id: i32,
        discord_id: &str,
    ) -> Result<Option<Match>, Self::Error>;

    /// Retrieves all matches associated with a tournament.
    ///
    /// Pass in a None for the round number to retrieve all matches for the tournament.
    async fn get_matches_by_tournament(
        &self,
        tournament_id: i32,
        round: impl Into<Option<i32>>,
    ) -> Result<Vec<Match>, Self::Error>;

    // async fn get_battle_record(&self, match_id: &str) -> Result<Option<BattleRecord>, Self::Error>;
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

        let db_url = match std::env::var("DATABASE_URL") {
            Ok(url) => url,
            Err(_) => {
                return Err(BotError::msg("DATABASE_URL environment variable not found"));
            }
        };
        let pool = PgPool::connect(db_url.as_str()).await?;
        info!("Successfully connected to the database.");

        Ok(PgDatabase { pool })
    }

    async fn migrate(&self) -> Result<(), Self::Error> {
        sqlx::migrate!("./migrations").run(&self.pool).await?;
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
        log_channel_id: &str,
        announcement_channel_id: &str,
    ) -> Result<(), Self::Error> {
        sqlx::query!(
            r#"
            INSERT INTO config (guild_id, marshal_role_id, log_channel_id, announcement_channel_id)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (guild_id)
            DO UPDATE SET
                marshal_role_id = $2,
                log_channel_id = $3,
                announcement_channel_id = $4
            "#,
            guild_id,
            marshal_role_id,
            log_channel_id,
            announcement_channel_id
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

    async fn create_user(&self, user: &User) -> Result<(), Self::Error> {
        sqlx::query!(
            r#"
            INSERT INTO users (discord_id, discord_name, player_tag, player_name, icon, trophies, brawlers)
            VALUES ($1, $2, $3, $4, $5, $6, $7::jsonb)
            ON CONFLICT (discord_id)
            DO UPDATE SET
                player_tag = $2
            "#,
            user.discord_id,
            user.discord_name,
            user.player_tag,
            user.player_name,
            user.icon,
            user.trophies,
            serde_json::to_value(user.brawlers.clone())?
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn delete_user(&self, discord_id: &str) -> Result<(), Self::Error> {
        sqlx::query!(
            r#"
            DELETE FROM users
            WHERE discord_id = $1
            "#,
            discord_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_player_by_discord_id(
        &self,
        discord_id: &str,
    ) -> Result<Option<Player>, Self::Error> {
        let user = sqlx::query_as!(
            Player,
            r#"
            SELECT discord_id, player_tag FROM users WHERE discord_id = $1
            LIMIT 1
            "#,
            discord_id
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(user)
    }

    async fn get_user_by_player(&self, player: Player) -> Result<Option<User>, Self::Error> {
        let user = sqlx::query_as!(
            User,
            r#"
           SELECT *
            FROM users 
            WHERE discord_id = $1 
                AND player_tag = $2
            LIMIT 1
            "#,
            player.discord_id,
            player.player_tag
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(user)
    }

    async fn get_player_by_player_tag(
        &self,
        player_tag: &str,
    ) -> Result<Option<Player>, Self::Error> {
        let user = sqlx::query_as!(
            Player,
            r#"
            SELECT discord_id, player_tag FROM users WHERE player_tag = $1
            LIMIT 1
            "#,
            player_tag
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    async fn create_tournament(
        &self,
        guild_id: &str,
        name: &str,
        tournament_id: impl Into<Option<i32>>,
        role_id: String,
        announcement_channel_id: &str,
        notification_channel_id: &str,
        wins_required: i32,
    ) -> Result<i32, Self::Error> {
        let timestamp_time = chrono::offset::Utc::now().timestamp();

        let tournament_id = match tournament_id.into() {
            None => {
                sqlx::query!(
                    r#"
            INSERT INTO tournaments (guild_id, name, created_at, rounds, current_round, tournament_role_id, announcement_channel_id, notification_channel_id, wins_required)
            VALUES ($1, $2, $3, 0, 0, $4, $5, $6, $7)
            RETURNING tournament_id
            "#,
                    guild_id,
                    name,
                    timestamp_time,
                    role_id,
                    announcement_channel_id,
                    notification_channel_id,
                    wins_required
                )
                .fetch_one(&self.pool)
                .await?
                .tournament_id
            }
            Some(custom_id) => {
                sqlx::query!(
                    r#"
            INSERT INTO tournaments (guild_id, name, created_at, tournament_id, rounds, current_round, tournament_role_id, announcement_channel_id, notification_channel_id)
            VALUES ($1, $2, $3, $4, 0, 0, $5, $6, $7)
            ON CONFLICT (tournament_id) DO NOTHING
            "#,
                    guild_id,
                    name,
                    timestamp_time,
                    custom_id,
                    role_id,
                    announcement_channel_id,
                    notification_channel_id
                )
                .execute(&self.pool)
                .await?;
                custom_id
            }
        };

        Ok(tournament_id)
    }

    async fn set_tournament_status(
        &self,
        tournament_id: i32,
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

    async fn get_tournament(
        &self,
        guild_id: &str,
        tournament_id: i32,
    ) -> Result<Option<Tournament>, Self::Error> {
        let tournament = sqlx::query_as!(
            Tournament,
            r#"
            SELECT tournament_id, guild_id, name, status as "status: _", rounds, current_round, created_at, start_time, map, tournament_role_id, wins_required, announcement_channel_id, notification_channel_id
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
            SELECT tournament_id, guild_id, name, status as "status: _", rounds, current_round, created_at, start_time, map, wins_required, tournament_role_id, announcement_channel_id, notification_channel_id
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
            SELECT tournament_id, guild_id, name, status as "status: _", rounds, current_round, created_at, start_time, map, wins_required, tournament_role_id, announcement_channel_id, notification_channel_id
            FROM tournaments WHERE guild_id = $1 AND (status != 'inactive')
            "#,
            guild_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(tournaments)
    }

    async fn get_player_active_tournaments(
        &self,
        guild_id: &str,
        discord_id: &str,
    ) -> Result<Vec<Tournament>, Self::Error> {
        let tournaments = sqlx::query_as!(
            Tournament,
            r#"
            SELECT tournaments.tournament_id, tournaments.guild_id, tournaments.name, tournaments.status as "status: _", tournaments.rounds, tournaments.current_round, tournaments.created_at, tournaments.start_time, tournaments.map, tournaments.wins_required, tournaments.tournament_role_id, tournaments.announcement_channel_id, tournaments.notification_channel_id
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

    async fn delete_tournament(&self, tournament_id: i32) -> Result<(), Self::Error> {
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

    async fn enter_tournament(
        &self,
        tournament_id: i32,
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

    async fn exit_tournament(
        &self,
        tournament_id: &i32,
        discord_id: &str,
    ) -> Result<(), Self::Error> {
        sqlx::query!(
            r#"
            DELETE FROM tournament_players
            WHERE tournament_id = $1 AND discord_id = $2
            "#,
            tournament_id,
            discord_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get_active_tournaments_from_player(
        &self,
        discord_id: &str,
    ) -> Result<Vec<Tournament>, Self::Error> {
        let tournament = sqlx::query_as!(
            Tournament,
            r#"
            SELECT t.tournament_id, t.guild_id, t.name, t.status as "status: _", t.rounds, t.current_round, t.created_at, t.start_time, t.map, t.wins_required, t.tournament_role_id, t.announcement_channel_id, t.notification_channel_id
            FROM tournaments AS t 
            JOIN tournament_players AS tp
            ON tp.tournament_id = t.tournament_id
            WHERE tp.discord_id = $1
            AND t.status != 'inactive';
            "#,
            discord_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(tournament)
    }

    async fn get_tournament_players(&self, tournament_id: i32) -> Result<Vec<Player>, Self::Error> {
        let players = sqlx::query_as!(
            Player,
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

    async fn set_rounds(&self, tournament_id: i32, rounds: i32) -> Result<(), Self::Error> {
        sqlx::query!(
            r#"
                UPDATE tournaments
                SET rounds = $1
                WHERE tournament_id = $2
                "#,
            rounds,
            tournament_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn next_round(&self, tournament_id: i32) -> Result<(), Self::Error> {
        sqlx::query!(
            r#"
                UPDATE tournaments
                SET current_round = current_round + 1
                WHERE tournament_id = $1
                "#,
            tournament_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn create_match(
        &self,
        tournament_id: i32,
        round: i32,
        sequence_in_round: i32,
        player_1_type: PlayerType,
        player_2_type: PlayerType,
        discord_id_1: Option<&str>,
        discord_id_2: Option<&str>,
    ) -> Result<(), Self::Error> {
        let match_id = Match::generate_id(tournament_id, round, sequence_in_round);

        sqlx::query!(
            r#"
            INSERT INTO matches (match_id, tournament_id, round, sequence_in_round, player_1_type, player_2_type, discord_id_1, discord_id_2, player_1_ready, player_2_ready, winner)
            VALUES ($1, $2, $3, $4, $5::player_type, $6::player_type, $7, $8, false, false, NULL)
            ON CONFLICT (match_id) DO NOTHING
            "#,
            match_id,
            tournament_id,
            round,
            sequence_in_round,
            player_1_type as PlayerType,
            player_2_type as PlayerType,
            discord_id_1,
            discord_id_2,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn set_ready(
        &self,
        match_id: &str,
        player_number: &PlayerNumber,
    ) -> Result<(), Self::Error> {
        match player_number {
            PlayerNumber::Player1 => {
                sqlx::query!(
                    r#"
                    UPDATE matches
                    SET player_1_ready = true
                    WHERE match_id = $1
                "#,
                    match_id
                )
                .execute(&self.pool)
                .await?
            }
            PlayerNumber::Player2 => {
                sqlx::query!(
                    r#"
                UPDATE matches
                SET player_2_ready = true
                WHERE match_id = $1
                "#,
                    match_id
                )
                .execute(&self.pool)
                .await?
            }
        };

        Ok(())
    }

    async fn set_winner(
        &self,
        match_id: &str,
        player_number: PlayerNumber,
    ) -> Result<(), Self::Error> {
        sqlx::query!(
            r#"
            UPDATE matches
            SET winner = $1::player_number
            WHERE match_id = $2
            "#,
            player_number as PlayerNumber,
            match_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_match_by_id(&self, match_id: &str) -> Result<Option<Match>, Self::Error> {
        let bracket = sqlx::query_as!(
            Match,
            r#"
            SELECT match_id, tournament_id, round, sequence_in_round, player_1_type as "player_1_type: _", player_2_type as "player_2_type: _", discord_id_1, discord_id_2, player_1_ready, player_2_ready, winner as "winner: _"
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
        tournament_id: i32,
        discord_id: &str,
    ) -> Result<Option<Match>, Self::Error> {
        let bracket = sqlx::query_as!(
            Match,
            r#"
            SELECT match_id, tournament_id, round, sequence_in_round, player_1_type as "player_1_type: _", player_2_type as "player_2_type: _", discord_id_1, discord_id_2, player_1_ready, player_2_ready, winner as "winner: _"
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
        tournament_id: i32,
        round: impl Into<Option<i32>>,
    ) -> Result<Vec<Match>, Self::Error> {
        let brackets = match round.into() {
            Some(round) => sqlx::query_as!(
                Match,
                r#"
                SELECT match_id, tournament_id, round, sequence_in_round, player_1_type as "player_1_type: _", player_2_type as "player_2_type: _", discord_id_1, discord_id_2, player_1_ready, player_2_ready, winner as "winner: _"
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
                SELECT match_id, tournament_id, round, sequence_in_round, player_1_type as "player_1_type: _", player_2_type as "player_2_type: _", discord_id_1, discord_id_2, player_1_ready, player_2_ready, winner as "winner: _"
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

    async fn set_map(&self, tournament_id: i32, map: &str) -> Result<(), Self::Error> {
        sqlx::query!(
            r#"
            UPDATE tournaments
            SET map = $1
            WHERE tournament_id = $2
            "#,
            map,
            tournament_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
    async fn add_records(&self, match_id: &str) -> Result<i64, Self::Error> {
        let query = sqlx::query!(
            r#"
                INSERT INTO battle_records (match_id)
                VALUES ($1)
                RETURNING record_id
                "#,
            match_id,
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(query.record_id)
    }

    async fn add_battle(&self, battle: Battle) -> Result<i64, Self::Error> {
        let query = sqlx::query!(
            r#"
            INSERT INTO battles (record_id, battle_time)
            VALUES ($1, $2)
            RETURNING id
            "#,
            battle.record_id,
            battle.battle_time,
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(query.id)
    }

    async fn add_battle_class(&self, battle_class: BattleClass) -> Result<i64, Self::Error> {
        let query = sqlx::query!(
            r#"
            INSERT INTO battle_classes (battle_id, mode, battle_type, result, duration, trophy_change, teams)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id
            "#,
            battle_class.battle_id,
            battle_class.mode as Mode,
            battle_class.battle_type as BattleType,
            battle_class.result as BattleResult,
            battle_class.duration,
            battle_class.trophy_change.unwrap_or(0),
            battle_class.teams, // teams as JSONB
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(query.id)
    }

    async fn add_event(&self, event: Event) -> Result<i64, Self::Error> {
        let query = sqlx::query!(
            r#"
            INSERT INTO events (mode, map)
            VALUES ($1, $2)
            RETURNING id
            "#,
            event.mode as Mode,
            event.map,
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(query.id)
    }

    async fn set_wins_required(
        &self,
        tournament_id: &i32,
        wins_required: &i32,
    ) -> Result<(), Self::Error> {
        sqlx::query!(
            r#"
            UPDATE tournaments
            SET wins_required = $1
            WHERE tournament_id = $2
            "#,
            wins_required,
            tournament_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
    // async fn get_battle_record(&self, match_id: &str) -> Result<Option<BattleRecord>, Self::Error> {
    //     let record = sqlx::query_as!(
    //         BattleRecord,
    //         r#"
    //         SELECT match_id, battle_log
    //         FROM battle_records
    //         WHERE match_id = $1
    //         LIMIT 1
    //         "#,
    //         match_id
    //     )
    //     .fetch_optional(&self.pool)
    //     .await?;

    //     Ok(record)
    // }
}
