import React, { useEffect, useState } from 'react';
import { SingleEliminationBracket } from '@g-loot/react-tournament-brackets';
import { MatchType, ParticipantType, Tournament } from '@/db/models';
import dynamic from 'next/dynamic';
import Head from 'next/head';
import Image from 'next/image';
import ErrorComponent from './error';
import Loading from './loading';
import { useRouter } from 'next/router';
interface SSEData {
  tournament: Tournament;
  matches: MatchType[];
}
const allMatchIds = new Set<string | number>();
const SSEClient: React.FC<{ path: string }> = ({ path }) => {
  const [data, setData] = useState<SSEData | null>(null);
  const [error, setError] = useState<string | null>(null);
 

  useEffect(() => {
    const eventSource = new EventSource(`${window.location.origin}${path}`);

    eventSource.onmessage = (event: MessageEvent) => {
      setData(JSON.parse(event.data));
    };

    eventSource.onerror = () => {
      setError('Connection to server lost.');
      eventSource.close();
    };

    return () => {
      eventSource.close();
    };
  }, [path]);

  if (error) {
    return (
      <ErrorComponent error={error} />
    )
  }

  if (!data) {
    return <Loading />;
  }

  const { tournament, matches } = data;
  if(allMatchIds.size == 0) matches.forEach((match) => allMatchIds.add(match.id));


  return (
    <div className='w-full h-full flex flex-col'>
      <Head>
        <meta property="og:title" content="Discord Brawl Cup" />
        <meta property="og:description" content={`View live result of ${tournament.name} here\nRound: ${tournament.current_round}`} />
      </Head>
      <div className='w-full flex-none'>
        <DisplayTournament tournament={tournament} />
      </div>
      <div className="w-full flex justify-center items-center py-1">
        <FindMatch/>
      </div>
      <div className='w-full grow overflow-y-auto overflow-x-auto'>
        {matches.length > 0 ? <TournamentSection matches={matches} /> : tournament.status === 'pending' ? <Pending /> : <div className='w-full h-full text-center'>No matches available</div>}
      </div>
    </div>
  );
};

const Pending: React.FC = () => {
  return (
    <div className='w-full h-full text-center'>
      The tournament has not started yet. Please stay tuned for more information.
    </div>
  )
}

const TournamentSection: React.FC<{ matches: MatchType[] }> = ({ matches }) => {
  const getParticipants = (participants: ParticipantType[]): { topParticipant: ParticipantType | null, bottomParticipant: ParticipantType | null } => {
    const [topParticipant, bottomParticipant] = participants;
    return {
      topParticipant: topParticipant ?? null,
      bottomParticipant: bottomParticipant ?? null,
    };
  };
  return (
    <SingleEliminationBracket
      matches={matches}
      options={{
        style: {
          roundHeader: {
            backgroundColor: '#FFD700',
            fontFamily: "LilitaOne-Regular",
          },
          connectorColor: '#FFD700',
        },
      }}
      svgWrapper={({ children }) => (
        children
      )}
      matchComponent={({ match }: { match: MatchType }) => {
        const { topParticipant, bottomParticipant } = getParticipants(match.participants);
        let startTime: string;
        if (!match.startTime) { // startTime: null
          startTime = "";
        }
        else if (isNaN(parseInt(match.startTime))) { // startTime: string
          startTime = match.startTime
        } else {
          startTime = new Date(parseInt(match.startTime) * 1000).toLocaleString();
        }
        return (
          <div className='relative w-full' id={`${match.id}`}>
            <p>{`${match.id}-${startTime}`}</p>
            <div className="flex flex-col">
              {topParticipant && topParticipant?.id !== "0" ? (
                <div className="relative flex items-center">
                  <Image
                    loader={() => topParticipant.iconUrl}
                    src="icon.png"
                    width={32}
                    height={32}
                    alt={topParticipant.name}
                    className={`w-8 mr-2 ${topParticipant.isWinner === true ? 'border-2 border-yellow-500' : topParticipant.isWinner === false ? 'grayscale' : ''}`}
                  />
                  <div className="flex-1">{topParticipant.name || 'Unknown'}</div>
                  <div>{topParticipant?.resultText || (topParticipant.isWinner ? 'Win' : 'Loss')}</div>
                </div>
              ) : (
                <TBD />
              )}

              <div className="h-px w-full bg-yellow-500"></div>

              {bottomParticipant && bottomParticipant?.id !== "0" ? (
                <div className="flex items-center">
                  <Image
                    loader={() => bottomParticipant.iconUrl}
                    src="icon.png"
                    width={32}
                    height={32}
                    alt={bottomParticipant.name}
                    className={`w-8 mr-2 ${bottomParticipant.isWinner === true ? 'border-2 border-yellow-500' : bottomParticipant.isWinner === false ? 'grayscale' : ''}`}
                  />
                  <div className="flex-1">{bottomParticipant.name || 'Unknown'}</div>
                  <div>{bottomParticipant?.resultText || (bottomParticipant.isWinner ? 'Win' : 'Loss')}</div>
                </div>
              ) : (
                <TBD />
              )}
            </div>
          </div>

        );
      }}
    />
  )
};

const TBD: React.FC<{ bye?: boolean }> = ({ bye = false }) => {
  return (
    <div className="flex items-center">
      {!bye ? (
        <>
          <Image
            loader={() => "https://cdn.brawlify.com/profile-icons/regular/28000000.png"}
            src="tbd.png"
            alt="TBD"
            width={32}
            height={32}
            className="w-8 mr-2"
          />
          <div className="flex-1">TBD</div>
          <div>TBD</div>
        </>
      ) : (
        <div className="w-full h-8 invisible" />
      )}
    </div>
  );
};

const DisplayTournament: React.FC<{ tournament: Tournament }> = ({ tournament }) => {
  const LocalDate: React.FC<{ unix: number }> = ({ unix }) => {
    const [localDate, setLocalDate] = useState('');
    useEffect(() => {
      const date = new Date(unix);
      const formattedDate = new Intl.DateTimeFormat('default', {
        year: 'numeric',
        month: '2-digit',
        day: '2-digit',
      }).format(date);
      setLocalDate(formattedDate);
    }, [unix]);

    return <div>Date: {localDate}</div>;
  };
  const DynamicLocalDate = dynamic(() => Promise.resolve(LocalDate), {
    ssr: false,
  });
  return (
    <div>
      <p className='w-full text-center pt-3 text-4xl'>{tournament.name}</p>
      <div className='w-full flex'>
        <p className='w-1/2 text-center'>ID: {tournament.tournament_id}</p>
        <p className='w-1/2 text-center'>Current round: {tournament.current_round}</p>
      </div>
      <div className='w-full flex'>
        <div className='w-1/2 text-center'>
          <DynamicLocalDate unix={parseInt(tournament.created_at) * 1000} />
        </div>
        <p className='w-1/2 text-center'>Status: {tournament.status}</p>
      </div>
    </div>
  )
};

const FindMatch: React.FC = () => {
  const [searchMatchId, setSearchMatchId] = useState<string>('');
  const router = useRouter();

  const handleSearch = () => {
    if (searchMatchId) {
      router.push(`#${searchMatchId}`, undefined, { shallow: true });
    }
  };

  const validMatchId = () => {
    return allMatchIds.has(searchMatchId);
  }
  return(
    <>
      <p className='px-2'>Match ID:</p>
        <input
          type="text"
          className={`py-1 px-2 border-b-2 ${validMatchId() ? "border-b-green-500" : "border-b-red-500"} focus:outline-none`}
          value={searchMatchId}
          maxLength={15}
          onChange={(e) => setSearchMatchId(e.target.value)}
        />
        <button className={`px-1 ${validMatchId()?"text-black":"text-gray-500"}`}
          onClick={handleSearch} 
          disabled={!validMatchId()}>Go</button>
    </>
  )
}

export default SSEClient;
