import BracketService from '@/services/bracket';
import TournamentService from '@/services/tournament';
import { NextApiRequest, NextApiResponse } from 'next';

export default async function handler(req: NextApiRequest, res: NextApiResponse): Promise<void> {
    res.setHeader('Content-Type', 'text/event-stream');
    res.setHeader('Cache-Control', 'no-cache, no-transform');
    res.setHeader('Connection', 'keep-alive');
    res.setHeader('X-Accel-Buffering', 'no');

    res.flushHeaders();

    const tournamentId = req.query.tournamentId as string;

    if (!tournamentId) {
        res.status(400).write(`event: error\ndata: ${JSON.stringify({ error: 'Missing tournamentId' })}\n\n`);
        res.end();
        return;
    }

    const sendEvent = async () => {
        try {
            const [tournament, tournamentError] = await TournamentService.getTournamentById(tournamentId);
            if (tournamentError) throw new Error(tournamentError.message);

            const [matches, matchesError] = await BracketService.getBracket(tournament.guild_id, tournament.tournament_id);
            if (matchesError) throw new Error(matchesError.message);

            const payload = { tournament, matches };
            res.write(`data: ${JSON.stringify(payload)}\n\n`);
        } catch (err) {
            console.error('Error fetching data:', err);
            res.write(`event: error\ndata: ${JSON.stringify({ error: err.message })}\n\n`);
        }
    };

    sendEvent();
    const intervalId = setInterval(sendEvent, 10000);

    req.on('close', () => {
        clearInterval(intervalId);
        res.end();
    });
}