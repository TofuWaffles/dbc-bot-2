import { getAllTournaments } from '@/db/handlers';
import { NextApiRequest, NextApiResponse } from 'next';

export default async function handler(req: NextApiRequest, res: NextApiResponse) {
  const { guildId } = req.query;
  if (typeof guildId !== 'string') {
    return res.status(400).json({ error: 'Invalid guild ID' });
  }
  const [tournaments, error] = await getAllTournaments(guildId);
  if (error) {
    return res.status(500).json({ error: 'Failed to load tournaments' });
  }
  res.status(200).json(tournaments);
}