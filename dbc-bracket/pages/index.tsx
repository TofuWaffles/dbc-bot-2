import React, { FC } from 'react';
import { GetStaticProps } from 'next';
import Link from 'next/link';
import TournamentService from './services/tournament';
import { Tournament } from '@/db/models';

const Home: FC<{ tournaments: Tournament[] }> = ({ tournaments }) => {
  return (
    <div className="flex min-h-screen bg-black text-white">
      <aside className="w-64 bg-gray-900 p-6">
        <h2 className="text-white mb-4">Tournaments Available</h2>
        {tournaments.length === 0 ? (
          <p className="text-gray-400">No tournaments available</p>
        ) : (
          <ul>
            {tournaments.map((tournament) => (
              <li key={tournament.tournament_id} className="mb-2 text-gray-400 hover:text-white cursor-pointer">
                <Link href={`/tournaments/${tournament.tournament_id}`}>{tournament.name}</Link>
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

export const getStaticProps: GetStaticProps = async () => {
  const [result, error] = await TournamentService.getAllTournaments();  
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
