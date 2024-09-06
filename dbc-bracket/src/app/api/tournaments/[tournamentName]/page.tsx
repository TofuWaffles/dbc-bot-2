import getMatchData, { getTournamentByName } from '@/app/db/handlers';
import { NextApiRequest, NextApiResponse } from 'next';

export default async function handler(req: NextApiRequest, res: NextApiResponse) {
  const { tournamentName } = req.query;

  if (typeof tournamentName !== 'string') {
    return res.status(400).json({ error: 'Invalid tournament name' });
  }

  try {
    const tournament = await getTournamentByName(tournamentName as string);
    const matchData = await getMatchData(tournament.tournament_id);
    res.status(200).json(matchData);
  } catch (error) {
    console.error(error);
    res.status(500).json({ error: 'Failed to load tournament match data' });
  }
}