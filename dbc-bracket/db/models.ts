export enum PlayerType {
    Player,
    Dummy,
    Pending
};

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
    start_time: string,
    tournament_role_id: string,
    status: string,
    mode: string,
    map: string,
    wins_required: number,
    announcement_channel_id: string,
    notification_channel_id: string
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