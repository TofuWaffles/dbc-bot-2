import { pool } from "./db";
import { Match, Tournament, MatchType, MATCH_STATES, TournamentStatus } from "../../app/db/models";

export default async function getMatchData(tournamentId: number): Promise<MatchType[]> {
    try {
        const result = await pool.query({ text: 'SELECT * FROM matches WHERE tournament_id = ANY($1)', values: [tournamentId] });
        const matchData: Match[] = Object.assign({}, ...result.rows);
        let matches: MatchType[] = [];
        for (const match of matchData) {
            const nextMatchState = await getNextMatchState(match.match_id);
            const player1Tag = await getPlayerTagByUserId(match.discord_id_1);
            const player2Tag = await getPlayerTagByUserId(match.discord_id_2);
            matches.push({
                id: match.match_id,
                nextMatchId: match.match_id + 1,
                tournamentRoundText: match.round.toString(),
                startTime: Date.now().toString(),
                state: nextMatchState,
                participants: [
                    {
                        id: player1Tag,
                        resultText: match.winner === 0 ? 'WON' : 'DEFEATED',
                        isWinner: match.winner === 0 ? true : false,
                        name: player1Tag
                    },
                    {
                        id: player2Tag,
                        resultText: match.winner === 1 ? 'WON' : 'DEFEATED',
                        isWinner: match.winner === 1 ? true : false,
                        name: player2Tag
                    }
                ]
            });
        }
        return matches;
    } catch (error) {
        console.error(error);
        throw new Error("Error fetching match data");
    }
}

export async function getNextMatchState(matchId: string): Promise<string> {
    try {
        const result = await pool.query({ text: 'SELECT TOP 1 match_id FROM matches WHERE match_id = ANY($1)', values: [matchId + 1] });
        if (result.rows) return MATCH_STATES.DONE;
        else return MATCH_STATES.NO_PARTY;
    } catch (error) {
        console.error(error);
        throw new Error("Error retrieving next round data");
    }
}

export async function getPlayerTagByUserId(discordId: string): Promise<string> {
    try {
        const result = await pool.query({ text: 'SELECT player_tag FROM users WHERE discord_id = ANY($1)', values: [discordId] });
        return result.rows.toString();
    } catch (error) {
        console.error(error);
        throw new Error("Error retrieving player data");
    }
}

export async function getTournamentByName(name: string): Promise<Tournament> {
    try {
        const result = await pool.query({ text: 'SELECT * FROM tournaments WHERE name = ANY($1)', values: [name] });
        const tournament: Tournament = Object.assign({}, ...result.rows);
        return tournament;
    } catch (error) {
        console.error(error);
        throw new Error("Error retrieving tournament data");
    }
}

export async function getAllTournaments(guildId: string): Promise<Tournament[]> {
    try {
        const result = await pool.query({
            text: 'SELECT * FROM tournaments WHERE guild_id = ANY($1)',
            values: [`{${guildId}}`]
        });

        return result.rows;
    } catch (error) {
        console.error(error);
        throw new Error("Error retrieving all tournaments data");
    }
}