import { getAllTournaments } from '../../db/handlers';
import { NextApiRequest, NextApiResponse } from 'next';

export default async function handler(req: NextApiRequest, res: NextApiResponse) {
  const { guildId } = req.query;

  if (typeof guildId !== 'string') {
    return res.status(400).json({ error: 'Invalid guild ID' });
  }

  try {
    console.log(guildId);
    const tournaments = await getAllTournaments(guildId);
    res.status(200).json(tournaments);
  } catch (error) {
    console.error(error);
    res.status(500).json({ error: 'Failed to load tournaments' });
  }
}