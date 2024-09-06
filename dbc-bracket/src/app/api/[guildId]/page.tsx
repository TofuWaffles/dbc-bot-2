import { getAllTournaments } from '@/app/db/handlers';
import { NextResponse } from 'next/server';

export async function GET({ params }: { params: { guildId: string } }) {
  const { guildId } = params;

  if (typeof guildId !== 'string') {
    return NextResponse.json({ error: 'Invalid guild ID' }, { status: 400 });
  }

  try {
    const tournaments = await getAllTournaments(guildId);

    return NextResponse.json(tournaments, { status: 200 });
  } catch (error) {
    console.error(error);
    return NextResponse.json({ error: 'Failed to load tournaments' }, { status: 500 });
  }
}

export default GET;