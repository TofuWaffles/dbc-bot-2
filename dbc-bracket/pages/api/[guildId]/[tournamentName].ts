import getMatchData, { getTournamentByNameAndGuildId } from '../../../db/handlers';
import { NextApiRequest, NextApiResponse } from 'next';

export default async function handler(req: NextApiRequest, res: NextApiResponse) {
  const { guildId, tournamentName } = req.query;

  if (typeof guildId !== 'string' || typeof tournamentName !== 'string') {
    return res.status(400).json({ error: 'Invalid guild ID or tournament name' });
  }

  try {
    const tournament = await getTournamentByNameAndGuildId(tournamentName, guildId);
    const matchData = await getMatchData(tournament[0].tournament_id);
    res.status(200).json(matchData);
  } catch (error) {
    console.error(error);
    res.status(500).json({ error: 'Failed to load tournament match data' });
  }
}