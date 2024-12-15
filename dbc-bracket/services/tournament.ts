import axios from "axios";
import "@/utils"
import { getAllAvailableTournaments, getTournamentById, getAllTournamentsInAGuild } from "@/db/handlers";

const TournamentService = {
  getAllTournamentsInAGuild,
  getAllAvailableTournaments,
  getTournamentById,
};
export default TournamentService;