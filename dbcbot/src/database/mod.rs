use crate::info;
use crate::BotError;
use anyhow::anyhow;
use models::*;
use poise::serenity_prelude::RoleId;
use sqlx::PgPool;
use tokio::join;
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

/// The Postgres database used for the DBC tournament system.
#[derive(Debug)]
pub struct PgDatabase {
    pub pool: PgPool,
}

impl PgDatabase {
    pub async fn connect() -> Result<Self, BotError> {
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

    pub async fn migrate(&self) -> Result<(), BotError> {
        sqlx::migrate!("./migrations").run(&self.pool).await?;
        Ok(())
    }
}

pub trait ConfigDatabase {
    type Error;
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

    /// Retrieves the marshal role of a given guild from the database.
    async fn get_marshal_role(&self, guild_id: &str) -> Result<Option<RoleId>, Self::Error>;
}

impl ConfigDatabase for PgDatabase {
    type Error = BotError;
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

    async fn get_marshal_role(&self, guild_id: &str) -> Result<Option<RoleId>, Self::Error> {
        let role = sqlx::query!(
            r#"
            SELECT marshal_role_id FROM config WHERE guild_id = $1
            LIMIT 1
            "#,
            guild_id
        )
        .fetch_optional(&self.pool)
        .await?;
        let marshal = match role {
            Some(r) => r.marshal_role_id,
            None => return Err(anyhow!("No marshal role found")),
        };
        Ok(marshal.parse().ok())
    }
}
pub trait UserDatabase {
    async fn get_tournament_id(&self, discord_id: &str) -> Result<Option<i32>, Self::Error>;
    type Error;
    /// Adds a user to the database.
    async fn create_user(&self, user: &Player) -> Result<(), Self::Error>;

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
    async fn get_user_by_player(&self, player: Player) -> Result<Option<Player>, Self::Error>;

    /// Retrieves a user from the database with a given discord id.
    async fn get_user_by_discord_id(&self, discord_id: &str)
        -> Result<Option<Player>, Self::Error>;

    /// Sets the ready status of a player of a specified match to true.
    async fn set_ready(&self, match_id: &str, discord_id: &str) -> Result<(), Self::Error>;

    /// Sets the winner of a match
    async fn set_winner(
        &self,
        match_id: &str,
        discord_id: &str,
        score: &str,
    ) -> Result<(), Self::Error>;

    async fn get_current_match(&self, discord_id: &str) -> Result<Option<Match>, Self::Error>;
}

impl UserDatabase for PgDatabase {
    type Error = BotError;
    async fn create_user(&self, user: &Player) -> Result<(), Self::Error> {
        sqlx::query!(
            r#"
            INSERT INTO users (discord_id, discord_name, player_tag, player_name, icon, trophies, brawlers)
            VALUES ($1, $2, $3, $4, $5, $6, $7::jsonb)
            ON CONFLICT (discord_id)
            DO UPDATE SET
                discord_name = EXCLUDED.discord_name,
                player_tag = EXCLUDED.player_tag,
                player_name = EXCLUDED.player_name,
                icon = EXCLUDED.icon,
                trophies = EXCLUDED.trophies,
                brawlers = EXCLUDED.brawlers,
                deleted = false
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
            UPDATE users
            SET deleted = true
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
            SELECT discord_id, player_tag, discord_name, player_name, icon, trophies, brawlers, deleted
            FROM users
            WHERE discord_id = $1
            LIMIT 1
            "#,
            discord_id
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(user)
    }

    async fn get_user_by_player(&self, player: Player) -> Result<Option<Player>, Self::Error> {
        let user = sqlx::query_as!(
            Player,
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

    async fn get_user_by_discord_id(
        &self,
        discord_id: &str,
    ) -> Result<Option<Player>, Self::Error> {
        let user = sqlx::query_as!(
            Player,
            r#"
            SELECT *
            FROM users 
            WHERE discord_id = $1 
            LIMIT 1
            "#,
            discord_id,
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
            SELECT discord_id, player_tag, discord_name, player_name, icon, trophies, brawlers, deleted
            FROM users
            WHERE player_tag = $1
            LIMIT 1
            "#,
            player_tag
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    async fn set_ready(&self, match_id: &str, discord_id: &str) -> Result<(), Self::Error> {
        sqlx::query!(
            r#"
            UPDATE match_players
            SET ready = true
            WHERE match_id = $1 AND discord_id = $2
        "#,
            match_id,
            discord_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn set_winner(
        &self,
        match_id: &str,
        discord_id: &str,
        score: &str,
    ) -> Result<(), Self::Error> {
        sqlx::query!(
            r#"
            UPDATE matches
            SET winner = $1
            WHERE match_id = $2 AND score = $3
            "#,
            discord_id,
            match_id,
            score
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_tournament_id(&self, discord_id: &str) -> Result<Option<i32>, Self::Error> {
        let tournament_id = sqlx::query!(
            r#"
            SELECT tournament_id
            FROM tournament_players
            WHERE discord_id = $1
            LIMIT 1
            "#,
            discord_id
        )
        .fetch_optional(&self.pool)
        .await?
        .map(|row| row.tournament_id);

        Ok(tournament_id)
    }

    async fn get_current_match(&self, discord_id: &str) -> Result<Option<Match>, Self::Error> {
        let tournament_id = self
            .get_tournament_id(discord_id)
            .await?
            .ok_or_else(|| anyhow!("No tournament found for player"))?;
        let current_round = self.current_round(tournament_id).await?;
        let current_match = sqlx::query!(
            r#"
            SELECT 
                m.match_id, 
                m.winner, 
                m.score,
                m.start,
                m.end
            FROM 
                matches AS m
            INNER JOIN 
                match_players AS mp
            ON 
                m.match_id = mp.match_id
            WHERE 
                mp.discord_id = $1
                AND m.match_id LIKE $2
            ORDER BY 
                m.match_id DESC
            LIMIT 1
            "#,
            discord_id,
            format!("%.{}.%", current_round)
        )
        .fetch_optional(&self.pool)
        .await?
        .map(|row| Match {
            match_id: row.match_id,
            match_players: Vec::with_capacity(2),
            winner: row.winner,
            score: row.score,
            start: row.start,
            end: row.end,
        });
        match current_match {
            None => Ok(None),
            Some(mut cm) => {
                let mut players = sqlx::query_as!(
                    MatchPlayer,
                    r#"
                    SELECT 
                        mp.match_id,
                        mp.discord_id,
                        mp.player_type AS "player_type: PlayerType",
                        mp.ready
                    FROM 
                        match_players AS mp
                    WHERE 
                        mp.match_id = $1
                    "#,
                    cm.match_id
                )
                .fetch_all(&self.pool)
                .await?;
                cm.match_players.append(&mut players);
                Ok(Some(cm))
            }
        }
    }
}
pub trait TournamentDatabase {
    async fn current_round(&self, tournament_id: i32) -> Result<i32, Self::Error>;
    type Error;
    /// Creates a tournament in the database, returning the tournament id.
    async fn create_tournament(
        &self,
        guild_id: &str,
        name: &str,
        mode: &Mode,
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
    async fn set_map(&self, tournament_id: i32, map: &BrawlMap) -> Result<(), Self::Error>;

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
}

impl TournamentDatabase for PgDatabase {
    type Error = BotError;
    async fn create_tournament(
        &self,
        guild_id: &str,
        name: &str,
        mode: &Mode,
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
            INSERT INTO tournaments (guild_id, name, mode, created_at, rounds, current_round, tournament_role_id, announcement_channel_id, notification_channel_id, wins_required)
            VALUES ($1, $2, $3, $4, 0, 0, $5, $6, $7, $8)
            RETURNING tournament_id
            "#,
                    guild_id,
                    name,
                    *mode as Mode,
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
            INSERT INTO tournaments (guild_id, name, mode, created_at, tournament_id, rounds, current_round, tournament_role_id, announcement_channel_id, notification_channel_id)
            VALUES ($1, $2, $3, $4, 0, 0, $5, $6, $7, $8)
            ON CONFLICT (tournament_id) DO NOTHING
            "#,
                    guild_id,
                    name,
                    *mode as Mode,
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
        let tournament = sqlx::query!(
            r#"
            SELECT 
            t.tournament_id, 
                t.guild_id, 
                t.name, 
                t.status AS "status: TournamentStatus",
                t.rounds, 
                t.current_round, 
                t.created_at, 
                t.start_time, 
                t.mode AS "mode: Mode",
                t.tournament_role_id, 
                t.wins_required, 
                t.announcement_channel_id, 
                t.notification_channel_id, 
                b.id as "map_id", 
                b.name as "map_name"
            FROM 
                tournaments AS t
            INNER JOIN 
                brawl_maps AS b
            ON 
                t.map = b.id
            WHERE 
                t.guild_id = $1 AND t.tournament_id = $2
            ORDER BY 
                t.created_at DESC
            LIMIT 1;
            
        "#,
            guild_id,
            tournament_id,
        )
        .fetch_optional(&self.pool)
        .await?
        .map(|row| Tournament {
            tournament_id: row.tournament_id,
            guild_id: row.guild_id,
            name: row.name,
            status: row.status,
            rounds: row.rounds,
            current_round: row.current_round,
            created_at: row.created_at,
            start_time: row.start_time,
            mode: row.mode,
            map: BrawlMap {
                id: row.map_id,
                name: row.map_name,
            },
            wins_required: row.wins_required,
            tournament_role_id: row.tournament_role_id,
            announcement_channel_id: row.announcement_channel_id,
            notification_channel_id: row.notification_channel_id,
        });
        Ok(tournament)
    }

    async fn get_all_tournaments(&self, guild_id: &str) -> Result<Vec<Tournament>, Self::Error> {
        let tournaments = sqlx::query!(
            r#"
            SELECT 
                t.tournament_id, 
                t.guild_id, t.name, 
                t.status as "status: TournamentStatus", 
                t.rounds, t.current_round, 
                t.created_at, t.start_time, 
                t.mode as "mode: Mode", 
                t.wins_required, 
                t.tournament_role_id, 
                t.announcement_channel_id, 
                t.notification_channel_id, 
                bm.id as "map_id", 
                bm.name as "map_name"
            FROM tournaments t
            INNER JOIN brawl_maps bm 
            ON t.map = bm.id
            WHERE t.guild_id = $1
            ORDER BY t.created_at DESC
            "#,
            guild_id
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|row| Tournament {
            tournament_id: row.tournament_id,
            guild_id: row.guild_id,
            name: row.name,
            status: row.status,
            rounds: row.rounds,
            current_round: row.current_round,
            created_at: row.created_at,
            start_time: row.start_time,
            mode: row.mode,
            map: BrawlMap {
                id: row.map_id,
                name: row.map_name,
            },
            wins_required: row.wins_required,
            tournament_role_id: row.tournament_role_id,
            announcement_channel_id: row.announcement_channel_id,
            notification_channel_id: row.notification_channel_id,
        })
        .collect::<Vec<Tournament>>();

        Ok(tournaments)
    }

    async fn get_active_tournaments(&self, guild_id: &str) -> Result<Vec<Tournament>, Self::Error> {
        let tournaments = sqlx::query!(
            r#"
            SELECT
                t.tournament_id, 
                t.guild_id, t.name, 
                t.status as "status: TournamentStatus", 
                t.rounds, t.current_round, 
                t.created_at, t.start_time, 
                t.mode as "mode: Mode", 
                t.wins_required, 
                t.tournament_role_id, 
                t.announcement_channel_id, 
                t.notification_channel_id, 
                b.id as "map_id", 
                b.name as "map_name"
            FROM tournaments AS t
            INNER JOIN brawl_maps AS b 
            ON t.map = b.id
            WHERE t.guild_id = $1 AND t.status != 'inactive'
            "#,
            guild_id
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|row| Tournament {
            tournament_id: row.tournament_id,
            guild_id: row.guild_id,
            name: row.name,
            status: row.status,
            rounds: row.rounds,
            current_round: row.current_round,
            created_at: row.created_at,
            start_time: row.start_time,
            mode: row.mode,
            map: BrawlMap {
                id: row.map_id,
                name: row.map_name,
            },
            wins_required: row.wins_required,
            tournament_role_id: row.tournament_role_id,
            announcement_channel_id: row.announcement_channel_id,
            notification_channel_id: row.notification_channel_id,
        })
        .collect::<Vec<Tournament>>();

        Ok(tournaments)
    }

    async fn get_player_active_tournaments(
        &self,
        guild_id: &str,
        discord_id: &str,
    ) -> Result<Vec<Tournament>, Self::Error> {
        let tournaments = sqlx::query!(
            r#"
            SELECT
                t.tournament_id, 
                t.guild_id, t.name, 
                t.status as "status: TournamentStatus", 
                t.rounds, t.current_round, 
                t.created_at, t.start_time, 
                t.mode as "mode: Mode", 
                t.wins_required, 
                t.tournament_role_id, 
                t.announcement_channel_id, 
                t.notification_channel_id, 
                b.id as "map_id", 
                b.name as "map_name"
FROM tournaments AS t
INNER JOIN tournament_players AS tp ON t.tournament_id = tp.tournament_id
INNER JOIN brawl_maps AS b ON t.map = b.id
WHERE t.guild_id = $1 AND (t.status = 'pending' OR t.status = 'started') AND tp.discord_id = $2;
            "#,
            guild_id,
            discord_id,
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|row| Tournament {
            tournament_id: row.tournament_id,
            guild_id: row.guild_id,
            name: row.name,
            status: row.status,
            rounds: row.rounds,
            current_round: row.current_round,
            created_at: row.created_at,
            start_time: row.start_time,
            mode: row.mode,
            map: BrawlMap {
                id: row.map_id,
                name: row.map_name,
            },
            wins_required: row.wins_required,
            tournament_role_id: row.tournament_role_id,
            announcement_channel_id: row.announcement_channel_id,
            notification_channel_id: row.notification_channel_id,
        })
        .collect::<Vec<Tournament>>();
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
        let tournament = sqlx::query!(
            r#"
            SELECT
                t.tournament_id, 
                t.guild_id, t.name, 
                t.status as "status: TournamentStatus", 
                t.rounds, t.current_round, 
                t.created_at, t.start_time, 
                t.mode as "mode: Mode", 
                t.wins_required, 
                t.tournament_role_id, 
                t.announcement_channel_id, 
                t.notification_channel_id, 
                bm.id as "map_id", 
                bm.name as "map_name"
            FROM tournaments AS t 
            JOIN tournament_players AS tp
            ON tp.tournament_id = t.tournament_id
            JOIN brawl_maps AS bm
            ON t.map = bm.id
            WHERE tp.discord_id = $1
            AND t.status != 'inactive';
            "#,
            discord_id
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|row| Tournament {
            tournament_id: row.tournament_id,
            guild_id: row.guild_id,
            name: row.name,
            status: row.status,
            rounds: row.rounds,
            current_round: row.current_round,
            created_at: row.created_at,
            start_time: row.start_time,
            mode: row.mode,
            map: BrawlMap {
                id: row.map_id,
                name: row.map_name,
            },
            wins_required: row.wins_required,
            tournament_role_id: row.tournament_role_id,
            announcement_channel_id: row.announcement_channel_id,
            notification_channel_id: row.notification_channel_id,
        })
        .collect::<Vec<Tournament>>();

        Ok(tournament)
    }

    async fn get_tournament_players(&self, tournament_id: i32) -> Result<Vec<Player>, Self::Error> {
        let players = sqlx::query_as!(
            Player,
            r#"
            SELECT users.discord_id, users.discord_name, users.player_name, users.player_tag, users.icon, users.trophies, users.brawlers, users.deleted
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

    async fn set_map(&self, tournament_id: i32, map: &BrawlMap) -> Result<(), Self::Error> {
        sqlx::query!(
            r#"
            UPDATE tournaments
            SET map = $1
            WHERE tournament_id = $2
            "#,
            map.id,
            tournament_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
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

    async fn current_round(&self, tournament_id: i32) -> Result<i32, Self::Error> {
        let round = sqlx::query!(
            r#"
            SELECT current_round
            FROM tournaments
            WHERE tournament_id = $1
            "#,
            tournament_id
        )
        .fetch_one(&self.pool)
        .await?
        .current_round;

        Ok(round)
    }
}

pub trait MatchDatabase {
    type Error;
    /// Creates a match associated with a tournament.
    async fn create_match(
        &self,
        tournament_id: i32,
        round: i32,
        sequence_in_round: i32,
    ) -> Result<(), Self::Error>;

    /// Enter a player into a match
    async fn enter_match(
        &self,
        match_id: &str,
        discord_id: &str,
        player_type: PlayerType,
    ) -> Result<(), Self::Error>;

    /// Retrieves all players in a given match.
    async fn get_match_players(&self, match_id: &str) -> Result<Vec<MatchPlayer>, Self::Error>;

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
}

impl MatchDatabase for PgDatabase {
    type Error = BotError;
    async fn create_match(
        &self,
        tournament_id: i32,
        round: i32,
        sequence_in_round: i32,
    ) -> Result<(), Self::Error> {
        let match_id = Match::generate_id(tournament_id, round, sequence_in_round);
        let start = chrono::Utc::now().timestamp();
        sqlx::query!(
            r#"
            INSERT INTO matches (match_id, score, start)
            VALUES ($1, $2, $3)
            ON CONFLICT (match_id) DO NOTHING
            "#,
            match_id,
            "0-0",
            start,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn enter_match(
        &self,
        match_id: &str,
        discord_id: &str,
        player_type: PlayerType,
    ) -> Result<(), Self::Error> {
        sqlx::query!(
            r#"
            INSERT INTO match_players (match_id, discord_id, player_type, ready)
            VALUES ($1, $2, $3, false)
            ON CONFLICT (match_id, discord_id) DO NOTHING
            "#,
            match_id,
            discord_id,
            player_type as PlayerType
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_match_players(&self, match_id: &str) -> Result<Vec<MatchPlayer>, Self::Error> {
        // ORDER BY discord_id ensures we always get the players in the same order.
        let players = sqlx::query_as!(
            MatchPlayer,
            r#"
                SELECT match_id, discord_id, player_type as "player_type: PlayerType", ready
                FROM match_players
                WHERE match_id = $1
                ORDER BY discord_id
                "#,
            match_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(players)
    }

    async fn get_match_by_id(&self, match_id: &str) -> Result<Option<Match>, Self::Error> {
        let players = self.get_match_players(match_id).await?;
        let bracket = match sqlx::query!(
            r#"
            SELECT match_id, winner, score, start, "end"
            FROM matches
            WHERE match_id = $1
            ORDER BY SPLIT_PART(match_id, '.', 2)::int DESC
            LIMIT 1
            "#,
            match_id
        )
        .fetch_optional(&self.pool)
        .await?
        {
            Some(r) => Some(Match {
                match_id: r.match_id,
                match_players: players,
                winner: r.winner,
                score: r.score,
                start: r.start,
                end: r.end,
            }),
            None => None,
        };

        Ok(bracket)
    }

    async fn get_match_by_player(
        &self,
        tournament_id: i32,
        discord_id: &str,
    ) -> Result<Option<Match>, Self::Error> {
        let bracket = match sqlx::query!(
            r#"
            SELECT 
                match_id, 
                winner, 
                score, 
                start, 
                "end"
            FROM matches
            WHERE 
                SPLIT_PART(match_id, '.', 1)::int = $1 -- Extract and match the tournament part
                AND match_id IN (
                    SELECT match_id
                    FROM match_players
                    WHERE discord_id = $2
                )
            ORDER BY 
                SPLIT_PART(match_id, '.', 2)::int DESC -- Order by round part
            LIMIT 1
            "#,
            tournament_id,
            discord_id,
        )
        .fetch_optional(&self.pool)
        .await?
        {
            Some(r) => {
                let players = self.get_match_players(&r.match_id).await?;
                Some(Match {
                    match_id: r.match_id,
                    match_players: players,
                    winner: r.winner,
                    score: r.score,
                    start: r.start,
                    end: r.end,
                })
            }
            None => None,
        };

        Ok(bracket)
    }

    async fn get_matches_by_tournament(
        &self,
        tournament_id: i32,
        round: impl Into<Option<i32>>,
    ) -> Result<Vec<Match>, Self::Error> {
        // Temp struct necessary because sqlx Records save query metadata, making them incompatible
        // with match statements
        struct TempMatch {
            match_id: String,
            winner: Option<String>,
            score: String,
            start: Option<i64>,
            end: Option<i64>,
        }

        let records = match round.into() {
            Some(round) => {
                sqlx::query_as!(
                    TempMatch,
                    r#"
                    SELECT 
                        match_id, 
                        winner, 
                        score, 
                        start, 
                        "end"
                    FROM matches
                    WHERE 
                        SPLIT_PART(match_id, '.', 1)::int = $1 -- tournament part
                        AND SPLIT_PART(match_id, '.', 2)::int = $2 -- round part (convert to int if needed)
                    ORDER BY SPLIT_PART(match_id, '.', 3)::int -- sequence part
                "#,
                    tournament_id,
                    round
                )
                .fetch_all(&self.pool)
                .await?
            }
            None => {
                sqlx::query_as!(
                    TempMatch,
                    r#"
                    SELECT 
                        match_id, 
                        winner, 
                        score, 
                        start, 
                        "end"
                    FROM matches
                    WHERE 
                        SPLIT_PART(match_id, '.', 1)::int = $1 -- tournament part
                    ORDER BY 
                        SPLIT_PART(match_id, '.', 2)::int DESC, -- round part in descending order
                        SPLIT_PART(match_id, '.', 3)::int       -- sequence part
                    "#,
                    tournament_id
                )
                .fetch_all(&self.pool)
                .await?
            }
        };

        let mut brackets = Vec::new();
        for record in records {
            let players = self.get_match_players(&record.match_id).await?;
            brackets.push(Match {
                match_id: record.match_id,
                match_players: players,
                winner: record.winner,
                score: record.score,
                start: record.start,
                end: record.end,
            });
        }
        Ok(brackets)
    }
}
pub trait BattleDatabase {
    async fn get_battle(&self, record_id: i64) -> Result<Vec<Battle>, Self::Error>;
    async fn get_record(&self, match_id: &str) -> Result<Option<BattleRecord>, Self::Error>;
    async fn add_event(&self, event: &Event, battle_id: i64) -> Result<i64, Self::Error>;
    async fn add_battle_class(
        &self,
        battle_class: &BattleClass,
        battle_id: i64,
    ) -> Result<i64, Self::Error>;
    type Error;
    async fn add_record(&self, record: &BattleRecord) -> Result<i64, Self::Error>;
    async fn add_battle(&self, battle: &Battle, record_id: i64) -> Result<i64, Self::Error>;
}

impl BattleDatabase for PgDatabase {
    type Error = BotError;
    async fn add_record(&self, record: &BattleRecord) -> Result<i64, Self::Error> {
        let id = sqlx::query!(
            r#"
            INSERT INTO battle_records (record_id,match_id)
            VALUES ($1, $2)
            RETURNING record_id
            "#,
            record.record_id,
            record.match_id
        )
        .fetch_one(&self.pool)
        .await?
        .record_id;
        Ok(id)
    }

    async fn add_battle(&self, battle: &Battle, record_id: i64) -> Result<i64, Self::Error> {
        let query = sqlx::query!(
            r#"
            INSERT INTO battles (id, record_id, battle_time)
            VALUES ($1, $2, $3)
            RETURNING id
            "#,
            battle.battle_time,
            record_id,
            battle.battle_time,
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(query.id)
    }

    async fn add_battle_class(
        &self,
        battle_class: &BattleClass,
        battle_id: i64,
    ) -> Result<i64, Self::Error> {
        let query = sqlx::query!(
            r#"
            INSERT INTO battle_classes (battle_id, mode, battle_type, result, duration, trophy_change, teams)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id
            "#,
            battle_id,
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

    async fn add_event(&self, event: &Event, battle_id: i64) -> Result<i64, Self::Error> {
        let query = sqlx::query!(
            r#"
            INSERT INTO events (mode, map, battle_id)
            VALUES ($1, $2, $3)
            RETURNING id
            "#,
            event.mode as Mode,
            event.map.id,
            battle_id
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(query.id)
    }

    async fn get_record(&self, match_id: &str) -> Result<Option<BattleRecord>, Self::Error> {
        let record = sqlx::query!(
            r#"
            SELECT *
            FROM battle_records
            WHERE match_id = $1
            "#,
            match_id
        )
        .fetch_optional(&self.pool)
        .await?;
        let record_id = record
            .map(|r| r.record_id)
            .ok_or(anyhow!("No record found"))?;
        let battles = self.get_battle(record_id).await?;
        Ok(Some(BattleRecord {
            record_id,
            match_id: match_id.to_string(),
            battles,
        }))
    }

    async fn get_battle(&self, record_id: i64) -> Result<Vec<Battle>, Self::Error> {
        // Fetch all battles based on record_id
        let battles = sqlx::query!(
            r#"
            SELECT *
            FROM battles
            WHERE record_id = $1
            "#,
            record_id
        )
        .fetch_all(&self.pool)
        .await?;
        let battle_ids: Vec<i64> = battles.iter().map(|b| b.id).collect();
        let (battle_classes, events) = join!(
            sqlx::query!(
                r#"
                SELECT 
                    bc.id,
                    bc.battle_id,
                    bc.mode AS "mode: Mode",
                    bc.battle_type AS "battle_type: BattleType",
                    bc.result AS "result: BattleResult",
                    bc.duration,
                    bc.trophy_change,
                    bc.teams
                FROM battle_classes AS bc
                WHERE battle_id = ANY($1)
                "#,
                &battle_ids
            ).fetch_all(&self.pool),
            
            sqlx::query!(
                r#"
                SELECT 
                    e.id,
                    e.mode AS "mode: Mode",
                    e.map,
                    e.battle_id
                FROM events AS e
                WHERE battle_id = ANY($1)
                "#,
                &battle_ids
            ).fetch_all(&self.pool)
        );
        let battle_classes = battle_classes?;
        let events = events?;
        let mut map_ids: Vec<i64> = vec![];
        for event in events.iter(){
            let id: i32  = event.map.ok_or(anyhow!("No map found"))?;
            map_ids.push(id as i64);
        }

        let maps: Vec<BrawlMap> = sqlx::query_as!(
            BrawlMap,
            r#"
            SELECT bm.id, bm.name
            FROM events AS e
            JOIN brawl_maps AS bm
            ON e.map = bm.id
            WHERE e.battle_id = ANY($1)
            "#,
             &map_ids
        ).fetch_all(&self.pool).await?;
        let mut battles_with_details = Vec::new();
        for record in battles {
            let battle_class = battle_classes
                .iter()
                .find(|bc| bc.battle_id == record.id)
                .ok_or_else(|| anyhow!("Battle class not found for battle id: {}", record.id))?;
            let event = events
                .iter()
                .find(|e| e.battle_id.unwrap() == record.id)
                .ok_or_else(|| anyhow!("Event not found for battle id: {}", record.id))?;
            let battle = Battle {
                id: record.id,
                battle_time: record.battle_time,
                record_id: record.record_id,
                battle_class: BattleClass {
                    id: battle_class.id,
                    battle_id: battle_class.battle_id,
                    mode: battle_class.mode.clone(),
                    battle_type: battle_class.battle_type.clone(),
                    result: battle_class.result.clone(),
                    duration: battle_class.duration,
                    trophy_change: battle_class.trophy_change,
                    teams: battle_class.teams.clone(),
                },
                event: Event {
                    id: event.id,
                    mode: event.mode.unwrap() as Mode,
                    map: maps.iter().find(|m| m.id == event.map.unwrap_or(0)).unwrap().clone(),
                    battle_id: event.battle_id.unwrap(),
                },
            };
            battles_with_details.push(battle);
        }

        // Return the collected battles
        Ok(battles_with_details)
    }
}

#[allow(async_fn_in_trait)]
pub trait Database {
    type Error;

    async fn create_map(&self, map: &BrawlMap) -> Result<(), Self::Error>;

    // async fn get_battle_record(&self, match_id: &str) -> Result<Option<BattleRecord>, Self::Error>;
}

impl Database for PgDatabase {
    type Error = BotError;

    async fn create_map(&self, map: &BrawlMap) -> Result<(), Self::Error> {
        sqlx::query!(
            r#"
            INSERT INTO brawl_maps (id, name)
            VALUES ($1, $2)
            ON CONFLICT (name) DO NOTHING
            "#,
            map.id,
            map.name
        )
        .fetch_one(&self.pool)
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
