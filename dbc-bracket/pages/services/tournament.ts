import axios from "axios";
import "@/utils"
import { Result, Err, Ok } from "@/utils";
import { Tournament } from "@/db/models";
const getAllTournamentsInAGuild = async (guildId: string): Promise<Result<Tournament[]>> => {
  const [response, error] = await axios.get<Tournament[]>(`/api/${guildId}`).wrapper();
  if(error){
    return Err(error);
  }
  return Ok(response.data);
}

const getAllTournaments = async (): Promise<Result<Tournament[]>> => {
  console.log("Fetching tournaments here");
  const [response, error] = await axios.get<Tournament[]>("/api/").wrapper();
  if(error){
    return Err(error);
  }
  return Ok(response.data);
}

const TournamentService = {
  getAllTournamentsInAGuild,
  getAllTournaments
};
export default TournamentService;