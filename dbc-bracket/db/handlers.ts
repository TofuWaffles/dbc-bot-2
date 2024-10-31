import { QueryConfig, QueryResult } from "pg";
import { pool } from "./db";
import {
  Match,
  Tournament,
  MatchType,
  MATCH_STATES,
  TournamentStatus,
  APILink,
  MatchPlayer,
  PlayerService,
  Player,
  MatchService,
  DefaultService,
} from "./models";
import "../utils";
import { Result } from "../utils";

function unixTimestampToDateTimeDMY(unixTimestamp: number): string {
  const date = new Date(unixTimestamp * 1000);
  const day = date.getDate();
  const month = date.getMonth() + 1;
  const year = date.getFullYear();
  const hours = date.getHours();
  const minutes = date.getMinutes().toString().padStart(2, "0");
  return `${day}/${month}/${year} ${hours}:${minutes}`;
}

export default async function getMatchData(
  tournamentId: number
): Promise<Result<MatchType[]>> {
  const [matchData, error] = await getMatchesByTournamentId(tournamentId);
  if (error) {
    return [[], error];
  }
  const matches: MatchType[] = new Array(matchData.length);
  for (const match of matchData) {
    const [nextMatchState, error1] = await getNextMatchState(match.match_id);
    if (error1) {
      return [[], error1];
    }
    const [nextMatchId, error2] = await getNextMatch(match.match_id);
    if (error2) {
      return [[], error2];
    }
    const [playerData, error3] = await getPlayerNamesAndIdsByMatchId(match.match_id);
    if (error3) {
      return [[], error3];
    }
    const [[_, round, sequence], error4] = MatchService.metadata(match);
    if (error4) {
      return [[], error4];
    }
    matches.push({
      id: sequence,
      nextMatchId: nextMatchId,
      tournamentRoundText: `Round ${round}`,
      startTime: unixTimestampToDateTimeDMY(match.start),
      state: nextMatchState,
      participants: playerData.map((player) => ({
        id: player.discord_id,
        resultText: match.winner === player.discord_id ? "WON" : "DEFEATED",
        isWinner: match.winner === player.discord_id ? true : false,
        name: player.player_name,
        iconUrl: PlayerService.icon(player.icon),
      })),
    });
  }
  return [matches, null];
}

export async function getNextMatch(matchId: string): Promise<Result<number>> {
  const [result, error] = await pool
    .query({
      text: "SELECT match_id FROM matches WHERE match_id = ANY($1)",
      values: [
        `{${matchId.split(".")[0]}.${matchId.split(".")[1]}.${
          parseInt(matchId.split(".")[2]) + 1
        }}`,
      ],
    })
    .wrapper();
  if (error) {
    console.error(error);
    return [Number.MIN_SAFE_INTEGER, error];
  }
  return result.rows[0]?.match_id
    ? [parseInt(result.rows[0].match_id.split(".")[2]), null]
    : [Number.MIN_SAFE_INTEGER, null];
}

export async function getNextMatchState(matchId: string): Promise<Result<string>> {
  const [result, error] = await pool
    .query({
      text: "SELECT 1 match_id FROM matches WHERE match_id = ANY($1)",
      values: [
        `{${matchId.split(".")[0]}.${matchId.split(".")[1]}.${
          parseInt(matchId.split(".")[2]) + 1
        }}`,
      ],
    })
    .wrapper();
  if (error) {
    console.error(error);
    return ["", error];
  }
  return result.rows
    ? [MATCH_STATES.DONE, null]
    : [MATCH_STATES.NO_PARTY, null];
}

export async function getPlayerNamesAndIdsByMatchId(
  matchId: string
): Promise<Result<Player[]>> {
  const [result, error] = await pool
    .query({
      text: "SELECT discord_id FROM match_players WHERE match_id = $1",
      values: [matchId],
    })
    .wrapper();
  if (error) {
    console.error(error);
    return [[], error];
  }
  const discordIds: string[] = result.rows.map((row) => row.discord_id);
  if (discordIds.length === 0) {
    return [[], null];
  }
  const [users, err] = await pool
    .query({
      text: "SELECT discord_id, player_name, icon FROM users WHERE discord_id = ANY($1)",
      values: [discordIds],
    })
    .wrapper();
  if (err) {
    console.error(err);
    return [[], err];
  }
  const playerData: Player[] = users.rows.map((row) => ({
    discord_id: row.discord_id,
    player_name: row.player_name,
    icon: PlayerService.icon(row.icon),
  }));
  return playerData.length > 0 ? [playerData, null] : [[], null];
}

export async function getTournamentByNameAndGuildId(
  name: string,
  guildId: string
): Promise<Result<Tournament>> {
  const [result, err] = await pool
    .query<Tournament>({
      text: "SELECT * FROM tournaments WHERE name = ANY($1) AND guild_id = ANY($2)",
      values: [`{${name}}`, `{${guildId}}`],
    })
    .wrapper();
  if (err) {
    console.error(err);
    return [DefaultService.DefaultTournament, err];
  }
  return [result.rows[0], null];
}

export async function getMatchesByTournamentId(
  tournamentId: number
): Promise<Result<Match[]>> {
  const [result, err] = await pool
    .query<Match>({
      text: "SELECT * FROM matches WHERE match_id LIKE $1",
      values: [`${tournamentId}.%`],
    })
    .wrapper();
  if (err) {
    console.error(err);
    return [[], err];
  }
  return [result.rows, null];
}

export async function getAllTournaments(guildId: string): Promise<Result<Tournament[]>> {
  const [result, err] = await pool
    .query<Tournament>({
      text: "SELECT * FROM tournaments WHERE guild_id = ANY($1)",
      values: [`{${guildId}}`],
    })
    .wrapper();
  if (err) {
    console.error(err);
    return [[], err];
  }
  return [result.rows, null];
}
