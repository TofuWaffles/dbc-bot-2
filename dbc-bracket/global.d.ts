declare module '@g-loot/react-tournament-brackets' {
    import SingleEliminationBracket from '@g-loot/react-tournament-brackets/dist/cjs/bracket-single/single-elim-bracket';
    import DoubleEliminationBracket from '@g-loot/react-tournament-brackets/dist/cjs/bracket-double/double-elim-bracket';
    import Match from '@g-loot/react-tournament-brackets/dist/cjs/components/match/index';
    import { MATCH_STATES } from '@g-loot/react-tournament-brackets/dist/cjs/core/match-states';
    import SVGViewer from '@g-loot/react-tournament-brackets/dist/cjs/svg-viewer';
    import { createTheme } from '@g-loot/react-tournament-brackets/dist/cjs/themes/themes';
    export { BracketLeaderboardProps, CommonTreeProps, ComputedOptionsType, DoubleElimLeaderboardProps, MatchComponentProps, MatchType, OptionsType, ParticipantType, SingleElimLeaderboardProps, SvgViewerProps, ThemeType, } from '@g-loot/react-tournament-brackets/dist/cjs/types';
    export { SingleEliminationBracket, DoubleEliminationBracket, Match, MATCH_STATES, SVGViewer, createTheme, };    
}
