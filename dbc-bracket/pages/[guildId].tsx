"use client";
import { useRouter } from 'next/router';
import { ReactNode, useEffect, useState } from 'react';
import { encode } from 'querystring';
import Sidebar from "@/pages/components/sidebar"
import "@/utils"
import TournamentService from '@/services/tournament';
interface GuildPage {
  children: ReactNode;
}

const GuildPage: React.FC<GuildPage> = () => {
  const router = useRouter();
  const guildId = encode(router.query).split('=')[1];
  const [items, setItems] = useState<string[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (guildId) {
      const fetchData = async () => {
        const [tournaments, error] = await TournamentService.getAllTournamentsInAGuild(guildId);
        if (error) {
          console.error(error);
          setError('Failed to load match data');
          return;
        }
        setItems(tournaments.map((tournament: { name: string }) => tournament.name));
        setLoading(false);
      };
      fetchData();
    } else {
      setLoading(false);
    }
  }, [guildId]);

  if (loading) return <div className="flex justify-center items-center h-screen text-lg">Loading...</div>;
  if (error) return <div className="flex justify-center items-center h-screen text-lg text-red-500">{error}</div>;

  return (
    <div className="flex min-h-screen bg-gray-100">
      <Sidebar items={items} guildId={guildId} />
      <main className="flex-1 flex flex-col justify-center items-center">
        <div className="bg-white shadow-lg rounded-lg p-8 w-full max-w-md">
          <h1 className="text-2xl font-bold mb-6 text-center">Tournaments</h1>
          <ul className="space-y-4 text-center">
            {items.map((item, index) => (
              <li key={index} className="text-lg">
                <a
                  href="#"
                  onClick={() => router.push(`/bracket/${guildId}/${item}`)}
                  className="text-blue-600 hover:text-blue-800 transition-colors duration-200"
                >
                  {item}
                </a>
              </li>
            ))}
          </ul>
        </div>
      </main>
    </div>
  );
};

export default GuildPage;