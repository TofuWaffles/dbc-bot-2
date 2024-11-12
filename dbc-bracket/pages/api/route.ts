import { getAllAvailableTournaments } from "@/db/handlers";
import { NextApiRequest, NextApiResponse } from "next";
export default async function handler(_req: NextApiRequest, res: NextApiResponse) {
  console.debug("Fetching all tournaments");
  const [tournaments, error] = await getAllAvailableTournaments();
  if (error) {
    return res.status(500).json({ error: "Failed to load tournaments" });
  }
  res.status(200).json(tournaments);
}