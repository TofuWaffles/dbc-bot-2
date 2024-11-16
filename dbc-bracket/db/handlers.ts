import { pool } from "@/db/db";
import {
  Match,
  Tournament,
  MatchType,
  MATCH_STATES,
  PlayerService,
  Player,
  MatchService,
} from "@/db/models";
import "@/utils";
import { Err, Ok, Result } from "@/utils";

const cache: { [key: number]: {
  matches: MatchType[],
  idxCache: { [key: string]: number }
}} = {};

export default async function getMatchData(
  tournamentId: number
): Promise<Result<MatchType[]>> {
  const [matchData, error] = await getMatchesByTournamentId(tournamentId);
  if (error) {
    return [[], error];
  }
  const [matches, error1] = await getPreAllMatches(tournamentId);
  if (error1) {
    return [[], error1];
  }
 
  for (const match of matchData) {
    const [
      [nextMatchState, error1],
      [playerData, error3],
    ] = await Promise.all([
      getNextMatchState(match),
      getPlayerNamesAndIdsByMatchId(match.match_id),
    ]);
    if (error1 || error3) {
      return Err(error1 || error3);
    }
    const [[_, round, sequence], error4] = MatchService.metadata(match);
    if (error4) {
      return [[], error4];
    }
    const matchId = match.match_id;
    const idx = cache[tournamentId].idxCache[matchId];
    matches[idx] = {
      ...matches[idx],
      tournamentRoundText: `${round}`,
      startTime: match.end?String(match.end):"Not started yet",
      state: nextMatchState,
      participants: playerData.map((player) => ({
        id: player.discord_id,
        resultText: MatchService.getScore(match, player),
        isWinner: match.winner === player.discord_id,
        name: player.player_name,
        iconUrl: String(player.icon),
      })),
    }
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

export async function getTournamentByTournamentIdAndGuildId(
  tournamentId: string,
  guildId: string
): Promise<Result<Tournament | null>> {
  const [result, err] = await pool
    .query<Tournament>({
      text: "SELECT * FROM tournaments WHERE tournament_id = $1 AND guild_id = $2",
      values: [tournamentId, guildId],
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
  if (err) {
    console.error(err);
    return Err(err);
  }
  return Ok(result.rows);
}

export async function getAllAvailableTournaments(): Promise<Result<Tournament[]>> {
  const [result, err] = await pool
    .query<Tournament>({text: "SELECT * FROM tournaments WHERE status = 'started'"})
    .wrapper();
  if (err) {
    console.error(err);
    return Err(err);
  }
  return Ok(result.rows);
}

export async function getAllTournamentsInAGuild(guildId: string): Promise<Result<Tournament[]>> {
  const [result, err] = await pool
    .query<Tournament>({text: "SELECT * FROM tournaments WHERE guild_id = ANY($1)", values: [[guildId]]})
    .wrapper();
  if (err) {
    console.error(err);
    return Err(err);
  }
  return Ok(result.rows);
}

export async function getTournamentById(id: string): Promise<Result<Tournament>> {
  const [result, err] = await pool
    .query<Tournament>({text: "SELECT * FROM tournaments WHERE tournament_id = $1", values: [id]})
    .wrapper();
  if (err) {
    console.error(err);
    return Err(err);
  }
  return Ok(result.rows[0]);
}

export async function getPreAllMatches(tournamentId: number): Promise<Result<MatchType[]>> {
  if(cache[tournamentId]){
    return Ok(cache[tournamentId].matches);
  }
  const [roundRaw, err] = await pool
    .query<{rounds: number}>({text: "SELECT rounds FROM tournaments WHERE tournament_id = $1", values: [tournamentId]})
    .wrapper();
  if (err) {
    console.log("Raw:",roundRaw);
    console.error(err);
    return Err(err);
  }
  const [playerCount, err1] = await pool
    .query<{count: number}>({text: `
      SELECT COUNT(m.*) AS count
      FROM match_players AS m 
      INNER JOIN tournaments AS t 
      ON SPLIT_PART(m.match_id, '.', 1)::INTEGER = t.tournament_id
      WHERE t.tournament_id = $1
      `, values: [tournamentId]})
    .wrapper();
  if (err1) {
    console.error(err1);
    return Err(err1);
  }
  const rounds = roundRaw.rows[0].rounds;
  const players = playerCount.rows[0].count;
  const matchCount = (playerCount: number, round: number) => {
    const roundedUp = Math.pow(2, Math.ceil(Math.log2(playerCount)));
    return roundedUp >> (round+1)};
  const matches: MatchType[] = [];
  let idx = 0;
  let idxCache: { [key: string]: number } = {};
  for(let round=1; round<=rounds; round++){
    let count = matchCount(players, round);
    for(let sequence=1; sequence<=count; sequence++){
      const matchId = `${tournamentId}.${round}.${sequence}`;
      const nextMatchId = round < rounds?`${tournamentId}.${round+1}.${Math.floor((sequence + 1) / 2)}`:null;
      matches.push({
        id: matchId,
        nextMatchId: nextMatchId,
        tournamentRoundText: `${round}`,
        startTime: null,
        state: MATCH_STATES.WALK_OVER,
        participants: [],
      });
      idxCache[matchId] = idx;
      idx++;
    }
  }
  cache[tournamentId] = {matches, idxCache};
  return Ok(matches);
}