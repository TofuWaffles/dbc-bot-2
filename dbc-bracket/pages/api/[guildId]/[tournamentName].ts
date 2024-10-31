import getMatchData, { getTournamentByNameAndGuildId } from '../../../db/handlers';
import { NextApiRequest, NextApiResponse } from 'next';

export default async function handler(req: NextApiRequest, res: NextApiResponse) {
  const { guildId, tournamentName } = req.query;

  if (typeof guildId !== 'string' || typeof tournamentName !== 'string') {
    return res.status(400).json({ error: 'Invalid guild ID or tournament name' });
  }
  const [tournament, error] = await getTournamentByNameAndGuildId(tournamentName, guildId);
  if (error) {
    console.error(error);
    return res.status(500).json({ error: 'Failed to load tournament data' });
  }
  const [matchData, error2] = await getMatchData(tournament.tournament_id);
  if (error) {
    console.error(error2);
    return res.status(500).json({ error: 'Failed to load tournament match data' });
  }
  console.debug(matchData);
  console.debug(JSON.stringify(matchData));
  res.status(200).json(matchData);
}