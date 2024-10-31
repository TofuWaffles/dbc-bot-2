import { Result } from "../utils"
export enum PlayerType {
    Player,
    Dummy,
    Pending
};

export interface Player{
    discord_id: string,
    player_name: string,
    icon: number | string,
    type?: PlayerType
}

export interface MatchPlayer{
    discord_id: string,
    match_id: string,
    icon_id: number
}


export enum TournamentStatus {
    Pending = 'Open for registration',
    Started = 'In progress',
    Paused = 'Paused',
    Inactive = 'Inactive/Completed'
};

export interface Match {
    match_id: string,
    winner: string,
    score: string,
    start: number,
    end: number,
};

export interface Tournament {
    tournament_id: number,
    name: string,
    guild_id: string,
    rounds: number,
    current_round: number,
    created_at: string,
    status: string,
};

export type ParticipantType = {
    id: string | number;
    isWinner?: boolean;
    name?: string;
    status?: 'PLAYED' | 'NO_SHOW' | 'WALK_OVER' | 'NO_PARTY' | string | null;
    resultText?: string | null;
    iconUrl?: string;
    [key: string]: any;
};

export type MatchType = {
    id: number | string;
    href?: string;
    name?: string;
    nextMatchId: number | string | null;
    nextLooserMatchId?: number | string;
    tournamentRoundText?: string;
    startTime: string;
    state: 'PLAYED' | 'NO_SHOW' | 'WALK_OVER' | 'NO_PARTY' | string;
    participants: ParticipantType[];
    [key: string]: any;
};

export const MATCH_STATES = {
    PLAYED: "played",
    NO_SHOW: "no_show",
    WALK_OVER: "walk_over",
    NO_PARTY: "no_party",
    DONE: "done",
    SCORE_DONE: "score_done",
};


export interface APILink {
    "player": (tag: string) => string;
    "club": (tag: string) => string;
    "clubMember": (tag: string) => string;
}

const tournamentId = (match: Match): Result<number> => {
    const parts = match.match_id.split('.');
    const [result, error] = parts.get(0);
    if (error) {
        console.error(error);
        return [null, error];
    }
    const tryParse = parseInt(result);
    if(isNaN(tryParse)) {
        return [null, new Error(`Failed to parse sequence number: ${result}`)];
    } else{
        return [tryParse, null];
    }
}

const round = (match: Match): Result<number> => {
    const parts = match.match_id.split('.');
    const [result, error] = parts.get(1);
    if (error) {
        console.error(error);
        return [null, error];
    }
    const tryParse = parseInt(result);
    if(isNaN(tryParse)) {
        return [null, new Error(`Failed to parse round number: ${result}`)];
    } else{
        return [tryParse, null];
    }
}

const sequence = (match: Match): Result<number> => {
    const parts = match.match_id.split('.');
    const [result, error] = parts.get(2)
    if (error) {
        console.error(error);
        return [null, error];
    }
    const tryParse = parseInt(result);
    if(isNaN(tryParse)) {
        return [null, new Error(`Failed to parse sequence number: ${result}`)];
    } else{
        return [tryParse, null];
    }
}

const metadata = (match: Match): Result<[number, number, number]> => {
    const parts = match.match_id.split('.');
    const [rawId, error1] = parts.get(0);
    if (error1) {
        console.error(error1);
        return [null, error1];
    }
    const [rawRound, error2] = parts.get(1);
    if (error2) {
        console.error(error2);
        return [null, error2];
    }
    const [rawSequence, error3] = parts.get(2);
    if (error3) {
        console.error(error3);
        return [null, error3];
    }
    const [tournamentId, round, sequence ]= [parseInt(rawId), parseInt(rawRound), parseInt(rawSequence)];
    if(isNaN(tournamentId) || isNaN(round) || isNaN(sequence)) {
        return [null, new Error(`Failed to parse metadata: ${match.match_id}`)];
    }
    return [[tournamentId, round, sequence], null];
}

export const MatchService = {
    sequence,
    round,
    tournamentId,
    metadata
}

const icon = (iconId: string | number): string => {
    return `https://cdn.brawlify.com/profile-icons/regular/${iconId}.png`
}

export const PlayerService = {
    icon
};

const DefaultTournament: Tournament = {
    tournament_id: 0,
    name: "",
    guild_id: "",
    rounds: 0,
    current_round: 0,
    created_at: "",
    status: TournamentStatus.Inactive
}

const DefaultPlayer: Player = {
    discord_id: "",
    player_name: "",
    icon: 0
}

const DefaultMatch: Match = {
    match_id: "",
    winner: "",
    score: "",
    start: 0,
    end: 0
}

export const DefaultService = {
    DefaultTournament,
    DefaultPlayer,
    DefaultMatch
}