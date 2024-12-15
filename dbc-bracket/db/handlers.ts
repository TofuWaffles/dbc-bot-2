import { pool } from "@/db/db";
import {
  Match,
  Tournament,
  MatchType,
  MATCH_STATES,
  PlayerService,
  Player,
  MatchService,
  BaseMatch,
  ParticipantType,
} from "@/db/models";
import "@/utils";
import { Err, Ok, Result } from "@/utils";
const skeletonParticipants: ParticipantType = {
  id: "0",
  resultText: null,
  isWinner: null,
  name: "tbd",
  iconUrl: null
}
const cache: { [key: number]: {
  matches: MatchType[],
  idxCache: { [key: string]: number }
}} = {};

interface Cache{
  tournamentId: number,
  matches: MatchType[],
  players: Player[]
}



export default async function getMatchData(
  tournamentId: number
): Promise<Result<MatchType[]>> {
  const [matchData, error] = await getMatchesByTournamentId(tournamentId);
  if (error) {
    return [[], error];
  }
  const [matches, error1] = await getAllPreMatches(tournamentId);
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
        isWinner: match.winner === null ? null : match.winner === player.discord_id,
        name: player.player_name,
        iconUrl: String(player.icon),
      })),
    }
    if(matches[idx].participants.length==1){
      matches[idx].participants.push(skeletonParticipants)
    }
  }
  console.log(JSON.stringify(matches, null, 2));
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
    .query<Tournament>({text: "SELECT * FROM tournaments WHERE status = 'started' OR status = 'pending'"})
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

export async function getAllPreMatches(tournamentId: number): Promise<Result<MatchType[]>> {
  if (cache[tournamentId]) {
    return Ok(cache[tournamentId].matches);
  }

  const [roundRaw, err] = await pool
    .query<{rounds: number}>({
      text: `
        SELECT 
          rounds 
        FROM tournaments 
        WHERE tournament_id = $1
      `,
      values: [tournamentId]
    })
    .wrapper();
  if (err) {
    console.error(err);
    return Err(err);
  }

  const [matchesRaw, err1] = await pool
    .query<Match>({
      text: `
        SELECT *
        FROM matches
        WHERE match_id LIKE $1
        ORDER BY match_id
      `,
      values: [`${tournamentId}.%`]
    })
    .wrapper();

  if (err1) {
    console.error(err1);
    return Err(err1);
  }

  const totalRounds = roundRaw.rows[0].rounds;
  const matches: MatchType[] = [];
  for(const match of matchesRaw.rows) {
    const [[tournament_id, round, sequence], err2] = MatchService.metadata(match);
    if (err2) {
      return Err(err2);
    }
    const nextMatchId = round < totalRounds ? `${tournament_id}.${round + 1}.${Math.ceil(sequence / 2)}` : null;
    matches.push({
      id: match.match_id,
      nextMatchId: nextMatchId,
      tournamentRoundText: `Round ${round}`,
      startTime: String(match.start),
      participants: [skeletonParticipants, skeletonParticipants]
    });
  }
  const idxCache: { [key: string]: number } = {};
  matches.forEach((match, index) => {
    idxCache[match.id] = index;
  });

  cache[tournamentId] = { matches, idxCache };

  return Ok(matches);
}

async function getPlayersFromTournament(tournamentId: number): Promise<Result<Player[]>>{
  const [result, err] = await pool
    .query<Player>({
      text: `
        SELECT 
          u.discord_id,
          u.player_name,
          u.icon
        FROM users u
        JOIN tournament_players tp
        ON u.discord_id = tp.discord_id
        WHERE tp.tournament_id = $1
      `,
      values: [tournamentId]
    })
    .wrapper();
  if (err) {
    console.error(err);
    return Err(err);
  }
  return Ok(result.rows);
}