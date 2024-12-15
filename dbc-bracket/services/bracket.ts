import "@/utils"
import { Result, Err, Ok } from "@/utils";
import { MatchType, Tournament } from "@/db/models";
import getMatchData, { getTournamentByTournamentIdAndGuildId } from "@/db/handlers";
const getBracket = async(guildId: string, tournamentId: number): Promise<Result<MatchType[]>> => {
  const [tournament, error]: [Tournament | null, Error] = await getTournamentByTournamentIdAndGuildId(tournamentId.toString(), guildId);
  if (error) {
    console.error(error);
    return Err(error);
  }
  const [matchData, error2] = await getMatchData(tournament.tournament_id);
  if (error2) {
    console.error(error2);
    return Err(error2);
  }
  return Ok(matchData);
}

const BracketService = {
  getBracket
}
export default BracketService;