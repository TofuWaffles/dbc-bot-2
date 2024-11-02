import React, { FC } from 'react';
import { GetStaticProps } from 'next';

interface HomeProps {
  tournaments: string[];
}

const Home: FC<HomeProps> = ({ tournaments }) => {
  return (
    <div className="flex min-h-screen bg-black text-white">
      <aside className="w-64 bg-gray-900 p-6">
        <h2 className="text-white mb-4">Tournament Options</h2>
        <ul>
          {tournaments.map((tournament, index) => (
            <li key={index} className="mb-2 text-gray-400 hover:text-white cursor-pointer">
              {tournament}
            </li>
          ))}
        </ul>
      </aside>

      <main className="flex-1 flex justify-center items-center">
        <h1 className="text-4xl">Welcome to the Tournament!</h1>
      </main>
    </div>
  );
};

export const getStaticProps: GetStaticProps = async () => {
  const tournaments = ['Option 1', 'Option 2', 'Option 3'];

  return {
    props: {
      tournaments,
    },
  };
};

export default Home;