import axios from "axios";
import "@/utils"
import { Result, Err, Ok } from "@/utils";
import { MatchType } from "@/db/models";
import { baseUrl } from "@/db/db";
const getBracket = async(guildId: string, tournamentId: number): Promise<Result<MatchType[]>> => {
  const [response, error] = await axios.get<MatchType[]>(`${baseUrl}/api/${guildId}/${tournamentId}`).wrapper();
  if(error){
    return Err(error);
  }
  return Ok(response.data);
}

const BracketService = {
  getBracket
}
export default BracketService;