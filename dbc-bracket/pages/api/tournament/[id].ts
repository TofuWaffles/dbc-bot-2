import { getTournamentById } from "@/db/handlers";
import { NextApiRequest, NextApiResponse } from "next";

export default async function handler(req: NextApiRequest, res: NextApiResponse) {
  const { id } = req.query;
  if (typeof id !== 'string') {
    return res.status(400).json({ error: 'Invalid guild ID' });
  }
  const [tournaments, error] = await getTournamentById(id);
  if (error) {
    return res.status(500).json({ error: 'Failed to load tournaments' });
  }
  res.status(200).json(tournaments);
}
