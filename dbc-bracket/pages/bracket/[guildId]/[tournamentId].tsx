import SSEClient from '@/components/SSEClient';
import { useRouter } from 'next/router';
import { usePathname } from 'next/navigation';
import { GetServerSideProps } from 'next';
import TournamentService from '@/services/tournament';
import { encode } from 'querystring';
interface Metadata{
    name: string;
    id: string;
    currentRound: number;
  }
const BracketHome: React.FC<Metadata> = (metadata) => {
    const router = useRouter();
    const { tournamentId } = router.query;
    const pathname = usePathname();

    if (!tournamentId) {
        return <div>Invalid tournament ID</div>;
    }

    return <SSEClient path={`/api${pathname}`} metadata={metadata}/>;
};

export const getServerSideProps: GetServerSideProps = async ({ query }) => {
    const tournamentId = encode(query).split('=')[2]; // extract tournamentId from query
    console.log("Tournament id:",tournamentId);
    const [tournament, error] = await TournamentService.getTournamentById(tournamentId);
    if (error) {
      return { props: { error: 'Failed to load match data' } };
    }
    const metadata = {
        name: tournament.name,
        id: tournament.tournament_id,
        currentRound: tournament.current_round
    }
    return { props: { metadata } };
  }
  
export default BracketHome;