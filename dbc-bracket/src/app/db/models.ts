export enum PlayerType {
    Player,
    Dummy,
    Pending
};

export enum PlayerNumber {
    Player1,
    Player2
};

export enum TournamentStatus {
    Pending = 'Open for registration',
    Started = 'In progress',
    Paused = 'Paused',
    Inactive = 'Inactive/Completed'
};

export interface Match {
    match_id: string,
    tournament_id: number,
    round: number,
    sequence_in_round: number,
    player_1_type: PlayerType,
    player_2_type: PlayerType,
    discord_id_1: string,
    discord_id_2: string,
    player_1_ready: boolean
    player_2_ready: boolean
    winner: PlayerNumber
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
  