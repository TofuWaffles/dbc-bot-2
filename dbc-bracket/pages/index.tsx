import React, { FC } from 'react';
import Link from 'next/link';
import TournamentService from '@/services/tournament';
import { Tournament } from '@/db/models';
import { GetServerSideProps } from 'next';
import Head from 'next/head';


const Home: FC<{ tournaments: Tournament[] }> = ({ tournaments }) => {
  return (
    <div className="flex min-h-screen bg-black text-white">
      <Head>
        <title>Discord Brawl Cup</title>
        <meta name="viewport" content="width=device-width, initial-scale=1" />
        <meta name="description" content="Discord Brawl Cup is /r/BrawlStars Discord Server's in-house competition where players face each other in a 1v1 bracket-style tournament to win prizes!" />
      </Head>
      <aside className="w-64 bg-gray-900 p-6">
        <h2 className="text-white mb-4">Tournaments Available</h2>
        {tournaments.length === 0 ? (
          <p className="text-gray-400">No tournaments available</p>
        ) : (
          <ul>
            {tournaments.map((tournament) => (
              <li key={tournament.tournament_id} className="mb-2 text-gray-400 hover:text-white cursor-pointer">
                <Link href={`/bracket/${tournament.guild_id}/${tournament.tournament_id}`}>{tournament.name}</Link>
              </li>
            ))}
          </ul>
        )}
      </aside>

      <main className="flex-1 flex justify-center items-center">
        <h1 className="text-4xl">Welcome to the Tournament!</h1>
      </main>
    </div>
  );
};

export const getServerSideProps: GetServerSideProps<{tournaments: Tournament[]}> = async () => {
  const [result, error] = await TournamentService.getAllAvailableTournaments();  
  if (error) {
    console.error(error);
    return {
      props: {
        tournaments: [], // Return empty array on error
      },
    };
  }

  const tournaments = result;
  return {
    props: {
      tournaments,
    },
  };
};

export default Home;
