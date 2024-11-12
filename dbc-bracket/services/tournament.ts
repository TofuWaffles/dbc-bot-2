import axios from "axios";
import "@/utils"
import { Result, Err, Ok } from "@/utils";
import { Tournament } from "@/db/models";
import { baseUrl } from "@/db/db";
const getAllTournamentsInAGuild = async (guildId: string): Promise<Result<Tournament[]>> => {
  const [response, error] = await axios.get<Tournament[]>(`/api/${guildId}`).wrapper();
  if(error){
    return Err(error);
  }
  return Ok(response.data);
}

const getAllTournaments = async (): Promise<Result<Tournament[]>> => {
  const [response, error] = await axios.get<Tournament[]>(`${baseUrl}/api/`).wrapper();
  if (error) {
    return Err(error);
  }
  return Ok(response.data);
}

const getTournamentById = async (id: string): Promise<Result<Tournament>> => {
  const [response, error] = await axios.get<Tournament>(`${baseUrl}/api/tournament/${id}`).wrapper();
  if (error) {
    return Err(error);
  }
  return Ok(response.data);
}

const TournamentService = {
  getAllTournamentsInAGuild,
  getAllTournaments,
  getTournamentById,
};
export default TournamentService;