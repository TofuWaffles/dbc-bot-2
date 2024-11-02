import { pool } from "@/db/db";
import {
  Match,
  Tournament,
  MatchType,
  MATCH_STATES,
  PlayerService,
  Player,
  MatchService,
  DefaultService,
} from "@/db/models";
import "@/utils";
import { Err, Ok, Result } from "@/utils";

export default async function getMatchData(
  tournamentId: number
): Promise<Result<MatchType[]>> {
  const [matchData, error] = await getMatchesByTournamentId(tournamentId);
  if (error) {
    return [[], error];
  }
  const matches: MatchType[] = [];
  for (const match of matchData) {
    const [
      [nextMatchState, error1],
      [nextMatchId, error2],
      [playerData, error3],
    ] = await Promise.all([
      getNextMatchState(match),
      getNextMatch(match),
      getPlayerNamesAndIdsByMatchId(match.match_id),
    ]);
    if (error1 || error2 || error3) {
      return Err(error1 || error2 || error3);
    }
    const [[_, round, sequence], error4] = MatchService.metadata(match);
    if (error4) {
      return [[], error4];
    }
    matches.push({
      id: match.match_id,
      nextMatchId: nextMatchId,
      tournamentRoundText: `Round ${round}`,
      startTime: match.start,
      state: nextMatchState,
      participants: playerData.map((player) => ({
        id: player.discord_id,
        resultText: MatchService.getScore(match, player),
        isWinner: match.winner === player.discord_id,
        name: player.player_name,
        iconUrl: String(player.icon),
      })),
    });
  }
  return Ok(matches);
}

export async function getNextMatch(
  match: Match
): Promise<Result<string | null>> {
  const tryNextMatch = MatchService.getNextMatchId(match);
  const [result, error2] = await pool
    .query<{ exist: boolean }>({
      text: "SELECT COUNT(*)>0 AS exist FROM matches WHERE match_id = $1",
      values: [tryNextMatch],
    })
    .wrapper();
  if (error2) {
    console.error(error2);
    return Err(error2);
  }
  return Ok(result.rows[0]?.exist ? tryNextMatch : null);
}

export async function getNextMatchState(match: Match): Promise<Result<string>> {
  const nextMatchId: string = MatchService.getNextMatchId(match);
  const [result, error2] = await pool
    .query<{ exist: boolean }>({
      text: "SELECT COUNT(*)>0 AS exist FROM matches WHERE match_id = $1",
      values: [nextMatchId],
    })
    .wrapper();
  if (error2) {
    console.error(error2);
    return Err(error2);
  }
  return result.rows[0]?.exist
    ? Ok(MATCH_STATES.DONE)
    : Ok(MATCH_STATES.NO_PARTY);
}

export async function getPlayerNamesAndIdsByMatchId(
  matchId: string
): Promise<Result<Player[]>> {
  const [result, error1] = await pool
    .query({
      text: "SELECT discord_id FROM match_players WHERE match_id = $1",
      values: [matchId],
    })
    .wrapper();
  if (error1) {
    console.error(error1);
    return Err(error1);
  }
  const discordIds: string[] = result.rows.map((row) => row.discord_id);
  if (discordIds.length === 0) {
    return Ok([]);
  }
  const [users, error2] = await pool
    .query({
      text: "SELECT discord_id, player_name, icon FROM users WHERE discord_id = ANY($1)",
      values: [discordIds],
    })
    .wrapper();
  if (error2) {
    console.error(error2);
    return Err(error2);
  }
  const playerData: Player[] = users.rows.map((row) => ({
    discord_id: row.discord_id,
    player_name: row.player_name,
    icon: PlayerService.icon(row.icon),
  }));
  return playerData.length > 0 ? Ok(playerData) : Ok([]);
}

export async function getTournamentByNameAndGuildId(
  name: string,
  guildId: string
): Promise<Result<Tournament | null>> {
  const [result, err] = await pool
    .query<Tournament>({
      text: "SELECT * FROM tournaments WHERE name = $1 AND guild_id = $2",
      values: [name, guildId],
    })
    .wrapper();
  if (err) {
    console.error(err);
    return Err(err);
  }
  return Ok(result.rows.length > 0? result.rows[0]: null);
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
  console.debug("Result", result.rows);
  if (err) {
    console.error(err);
    return Err(err);
  }
  return Ok(result.rows);
}

export async function getAllTournaments(
  guildId: string
): Promise<Result<Tournament[]>> {
  const [result, err] = await pool
    .query<Tournament>({
      text: "SELECT * FROM tournaments WHERE guild_id = ANY($1)",
      values: [`{${guildId}}`],
    })
    .wrapper();
  if (err) {
    console.error(err);
    return Err(err);
  }
  return Ok(result.rows);
}
