declare module "@g-loot/react-tournament-brackets" {
  import SingleEliminationBracket from "@g-loot/react-tournament-brackets/dist/cjs/bracket-single/single-elim-bracket";
  import DoubleEliminationBracket from "@g-loot/react-tournament-brackets/dist/cjs/bracket-double/double-elim-bracket";
  import Match from "@g-loot/react-tournament-brackets/dist/cjs/components/match/index";
  import { MATCH_STATES } from "@g-loot/react-tournament-brackets/dist/cjs/core/match-states";
  import SVGViewer from "@g-loot/react-tournament-brackets/dist/cjs/svg-viewer";
  import { createTheme } from "@g-loot/react-tournament-brackets/dist/cjs/themes/themes";
  export {
    BracketLeaderboardProps,
    CommonTreeProps,
    ComputedOptionsType,
    DoubleElimLeaderboardProps,
    MatchComponentProps,
    MatchType,
    OptionsType,
    ParticipantType,
    SingleElimLeaderboardProps,
    SvgViewerProps,
    ThemeType,
  } from "@g-loot/react-tournament-brackets/dist/cjs/types";
  export {
    SingleEliminationBracket,
    DoubleEliminationBracket,
    Match,
    MATCH_STATES,
    SVGViewer,
    createTheme,
  };
}

declare global {
  type Result<T> = [NonNullable<T>, null | Error];
  interface Promise<T> {
    /**
     * Wraps the current Promise in a Result type.
     * This method transforms the Promise into a Result, allowing for easier error handling
     * and management of success states.
     *
     * @returns {Result<T>} - A Result representing the success or failure of the Promise.
     */
    wrapper(): Promise<Result<T>>;

    /**
     * Unwraps the current Promise, returning the resolved value if the Promise is fulfilled.
     * If the Promise is rejected, this method will throw an error.
     *
     * @throws {Error} - Throws an error if the Promise is rejected.
     * @returns {Promise<T>} - The original Promise that was unwrapped.
     */
    unwrap(): Promise<T>;

    /**
     * Unwraps the current Promise, returning the resolved value if the Promise is fulfilled.
     * If the Promise is rejected, this method will return the provided default value instead.
     *
     * @param {T} defaultValue - The value to return if the Promise is rejected.
     * @returns {Promise<T>} - A Promise that resolves to either the unwrapped value or the default value.
     */
    unwrapOr(defaultValue: T): Promise<T>;
  }

  interface Array<T> {
    get(index: number): Result<T>;
  }
}
export {};
