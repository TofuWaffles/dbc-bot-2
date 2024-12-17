import { GetServerSideProps } from 'next';
import { encode } from 'querystring';
import Sidebar from "@/components/sidebar";
import TournamentService from '@/services/tournament';
import Link from 'next/link';
import Custom404 from './404';

interface GuildPageProps {
  items: { id: string, name: string }[];
  guildId: string;
  error?: string;
}

const GuildPage: React.FC<GuildPageProps> = ({ items, guildId, error }) => {
  if (error) {
    return <div className="flex justify-center items-center h-screen text-lg text-red-500">{error}</div>;
  }
  if (!items.length ){
    return(
      <div className='w-full h-full justify-center items-center flex'>
        <Custom404/>
      </div>
    )
  }

  return (
    <div className="flex min-h-screen bg-gray-100">
      <Sidebar items={items} guildId={guildId} />
      <main className="flex-1 flex flex-col justify-center items-center">
        <div className="bg-white shadow-lg rounded-lg p-8 w-full max-w-md">
          <h1 className="text-2xl font-bold mb-6 text-center">Tournaments</h1>
          <ul className="space-y-4 text-center">
            {items.map((item, index) => (
              <li key={index} className="text-lg">
                <Link
                  href={`../bracket/${guildId}/${item.id}`}
                  className="text-blue-600 hover:text-blue-800 transition-colors duration-200"
                >
                  {item.name}
                </Link>
              </li>
            ))}
          </ul>
        </div>
      </main>
    </div>
  );
};

// Server-side data fetching
export const getServerSideProps: GetServerSideProps = async ({ query }) => {
  const guildId = encode(query).split('=')[1]; // extract guildId from query
  if (!guildId) {
    return { props: { error: 'No guild ID provided' } };
  }

  const [tournaments, error] = await TournamentService.getAllTournamentsInAGuild(guildId);
  if (error) {
    return { props: { error: 'Failed to load match data' } };
  }
  const items = tournaments.map((tournament) => {
    return {
      id: tournament.tournament_id,
      name: tournament.name,
    }
  });
  return { props: { items, guildId } };
}


export default GuildPage;
