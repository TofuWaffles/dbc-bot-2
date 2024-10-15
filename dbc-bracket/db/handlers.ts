import { pool } from "./db";
import { Match, Tournament, MatchType, MATCH_STATES, TournamentStatus, APILink } from "./models";

function unixTimestampToDateTimeDMY(unixTimestamp: number): string {
    const date = new Date(unixTimestamp * 1000);
    const day = date.getDate();
    const month = date.getMonth() + 1;
    const year = date.getFullYear();
    const hours = date.getHours();
    const minutes = date.getMinutes().toString().padStart(2, '0');
    return `${day}/${month}/${year} ${hours}:${minutes}`;
}

export default async function getMatchData(tournamentId: number): Promise<MatchType[]> {
    try {
        const matchData: Match[] = await getMatchesByTournamentId(tournamentId);
        let matches: MatchType[] = [];
        for (const match of matchData) {
            const nextMatchState = await getNextMatchState(match.match_id);
            const playerData = await getPlayerNamesAndIdsByMatchId(match.match_id);
            const player1Id = playerData[0].discord_id;
            const player2Id = playerData[1].discord_id;
            const player1Name = playerData[0].player_name;
            const player2Name = playerData[1].player_name;
            const player1Icon = playerData[0].icon;
            const player2Icon = playerData[1].icon;
            const round = parseInt(match.match_id.split('.')[1]);
            const sequenceInRound = parseInt(match.match_id.split('.')[2]);
            matches.push({
                id: sequenceInRound,
                nextMatchId: await getNextMatch(match.match_id),
                tournamentRoundText: `Round ${round}`,
                startTime: unixTimestampToDateTimeDMY(match.start),
                state: nextMatchState,
                participants: [
                    {
                        id: player1Id,
                        resultText: match.winner === player1Id ? 'WON' : 'DEFEATED',
                        isWinner: match.winner === player1Id ? true : false,
                        name: player1Name,
                        iconUrl: player1Icon
                    },
                    {
                        id: player2Id,
                        resultText: match.winner === player2Id ? 'WON' : 'DEFEATED',
                        isWinner: match.winner === player2Id ? true : false,
                        name: player2Name,
                        iconUrl: player2Icon
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


export async function getNextMatch(matchId: string): Promise<number> {
    try {
        const result = await pool.query({ text: 'SELECT match_id FROM matches WHERE match_id = ANY($1)', values: [`{${matchId.split('.')[0]}.${matchId.split('.')[1]}.${parseInt(matchId.split('.')[2]) + 1}}`] });
        if (result.rows[0]?.match_id) return parseInt(result.rows[0].match_id.split('.')[2]);
        else return undefined;
    } catch (error) {
        console.error(error);
        throw new Error("Error retrieving next round data");
    }
}

export async function getNextMatchState(matchId: string): Promise<string> {
    try {
        const result = await pool.query({ text: 'SELECT 1 match_id FROM matches WHERE match_id = ANY($1)', values: [`{${matchId.split('.')[0]}.${matchId.split('.')[1]}.${parseInt(matchId.split('.')[2]) + 1}}`] });
        if (result.rows) return MATCH_STATES.DONE;
        else return MATCH_STATES.NO_PARTY;
    } catch (error) {
        console.error(error);
        throw new Error("Error retrieving next round data");
    }
}

export async function getPlayerNamesAndIdsByMatchId(matchId: string): Promise<{ discord_id: string, player_name: string, icon: string }[]> {
    try {
        const matchPlayers = await pool.query({ text: 'SELECT discord_id FROM match_players WHERE match_id = $1', values: [matchId] });

        const discordIds: string[] = matchPlayers.rows.map(row => row.discord_id);

        if (discordIds.length === 0) {
            return undefined;
        }

        const users = await pool.query({ text: 'SELECT discord_id, player_name, icon FROM users WHERE discord_id = ANY($1)', values: [discordIds] });

        const playerData = users.rows.map(row => ({
            discord_id: row.discord_id,
            player_name: row.player_name,
            icon: `https://cdn-old.brawlify.com/profile/${row.icon}.png`
        }));

        if (playerData.length > 0) {
            return playerData;
        } else {
            return undefined;
        }
    } catch (error) {
        console.error(error);
        throw new Error("Error retrieving player data");
    }
}

export async function getTournamentByNameAndGuildId(name: string, guildId: string): Promise<Tournament> {
    try {
        const result = await pool.query({ text: 'SELECT * FROM tournaments WHERE name = ANY($1) AND guild_id = ANY($2)', values: [`{${name}}`, `{${guildId}}`] });
        return result.rows;
    } catch (error) {
        console.error(error);
        throw new Error("Error retrieving tournament data");
    }
}

export async function getMatchesByTournamentId(tournamentId: number): Promise<Match[]> {
    try {
        const result = await pool.query({
            text: 'SELECT * FROM matches WHERE match_id LIKE $1',
            values: [`${tournamentId}.%`]
        });
        return result.rows;
    } catch (error) {
        console.error(error);
        throw new Error("Error retrieving match data");
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