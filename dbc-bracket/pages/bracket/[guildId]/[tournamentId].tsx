
import { SingleEliminationBracket, SVGViewer } from '@g-loot/react-tournament-brackets';
import { MatchType, ParticipantType, Tournament } from '@/db/models';
import { useEffect, useState } from 'react';
import { useWindowSize } from '@uidotdev/usehooks';
import BracketService from '@/services/bracket';
import { GetServerSideProps } from 'next';
import Image from 'next/image';
import TournamentService from '@/services/tournament';
import dynamic from 'next/dynamic';

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

interface TournamentPage {
  tournament: Tournament
  matches: MatchType[]
}

const getParticipants = (participants: ParticipantType[]): { topParticipant: ParticipantType | null, bottomParticipant: ParticipantType | null } => {
  const [topParticipant, bottomParticipant] = participants;
  return {
    topParticipant: topParticipant ?? null,
    bottomParticipant: bottomParticipant ?? null,
  };
};
const TournamentPage: React.FC<TournamentPage> = ({ tournament, matches }) => {
  return (
    <div>
      <p className='w-full text-center pt-3 text-4xl'>{tournament.name}</p>
      <div className='w-full flex justify-evenly items-center'>
        <p>ID: {tournament.tournament_id}</p>
        <p>Current round: {tournament.current_round}</p>
        <DynamicLocalDate unix={parseInt(tournament.created_at) * 1000}/>
        <p>Status: {tournament.status}</p>
      </div>
      {matches.length > 0 ? <TournmanentSection matches={matches} /> : <div>Loading...</div>}
    </div>
  );
};

const TournmanentSection: React.FC<{ matches: MatchType[] }> = ({ matches }) => {
  const { width, height } = useWindowSize();
  useEffect(() => {
    console.log("matches", matches);
  }, [matches]);
  return (
    <div className="w-full h-full flex justify-center items-center p-20">
      <div className="w-full h-full">
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
          svgWrapper={({ children, ...props }) => (
            <SVGViewer
              // background="#000"
              // SVGBackground="#000"
              width={width}
              height={height}
              {...props}
            >
              {children}
            </SVGViewer>
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
              <div className='w-full h-full'>
                <p>{startTime || <>&nbsp;</>}</p>
                <div className="flex flex-col">
                  {topParticipant ? (
                    <div className="flex items-center">
                      <Image
                        loader={() => topParticipant.iconUrl}
                        src="icon.png"
                        alt={topParticipant.name}
                        className={`w-8 mr-2 ${topParticipant.isWinner ? 'border-2 border-yellow-500' : 'grayscale'}`}
                      />
                      <div className="flex-1">{topParticipant.name || 'Unknown'}</div>
                      <div>{topParticipant.resultText || (topParticipant.isWinner ? 'Win' : 'Loss')}</div>
                    </div>
                  ) : (
                    <TBD />
                  )}

                  <div className="h-px w-full bg-yellow-500"></div>

                  {bottomParticipant ? (
                    <div className="flex items-center">
                      <Image
                        loader={() => bottomParticipant.iconUrl}
                        src="icon.png"
                        alt={bottomParticipant.name}
                        className="w-8 mr-2"
                      />
                      <div className="flex-1">{bottomParticipant.name || 'Unknown'}</div>
                      <div>{bottomParticipant.resultText || (bottomParticipant.isWinner ? 'Win' : 'Loss')}</div>
                    </div>
                  ) : (
                    <TBD />
                  )}
                </div>
              </div>

            );
          }}
        // matchComponent={Match}
        />
      </div>
    </div>
  )
};

const TBD: React.FC = () => {
  return (
    <div className="flex items-center">
      <Image
        loader={() => "https://cdn.brawlify.com/profile-icons/regular/28000000.png"}
        src="tbd.png"
        alt="TBD"
        className="w-8 mr-2"
      />
      <div className="flex-1">TBD</div>
      <div>TBD</div>
    </div>
  )
}

export const getServerSideProps: GetServerSideProps = async (context) => {
  const { tournamentId } = context.params as { tournamentId: string };
  const [tournament, error] = await TournamentService.getTournamentById(tournamentId);
  if (error) {
    console.error(error);
    return {
      notFound: true,
    };
  }
  const [matches, err] = await BracketService.getBracket(tournament.guild_id, tournament.tournament_id);
  if (err) {
    console.error(err);
    return {
      notFound: true,
    };
  }
  return {
    props: {
      tournament,
      matches,
    },
  };
};


export default TournamentPage;