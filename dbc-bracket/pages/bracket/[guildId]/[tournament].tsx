"use client";
import { SingleEliminationBracket, SVGViewer, createTheme, Match } from '@g-loot/react-tournament-brackets';
import { MatchType, ParticipantType } from '@/db/models';
import { ReactNode, useEffect, useState } from 'react';
import { useRouter } from 'next/router';
import { useWindowSize } from '@uidotdev/usehooks';
import brawlStarsBackground from '@/../images/assets/battle_log_bg.png';
import defaultPlayerIcon from '@/../images/assets/28000000.png';

interface TournamentPage {
  children: ReactNode;
}

const getParticipants = (participants: ParticipantType[]): { topParticipant: ParticipantType | null, bottomParticipant: ParticipantType | null } => {
  const [topParticipant, bottomParticipant] = participants;
  return {
    topParticipant: topParticipant ?? null,
    bottomParticipant: bottomParticipant ?? null,
  };
};
const TournamentPage: React.FC<TournamentPage> = ({ children }) => {
  const router = useRouter();
  const [matches, setMatches] = useState<MatchType[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const { width, height } = useWindowSize();

  useEffect(() => {
    console.debug("width, height", width, height);
    if (router.isReady) {
      const { guildId, tournament } = router.query;

      if (guildId && tournament) {
        const fetchData = async () => {
          try {
            const response = await fetch(`/api/${guildId}/${tournament}`);
            if (!response.ok) {
              throw new Error('Network response was not ok');
            }
            const tournamentData = await response.json();
            console.log(tournamentData);
            setMatches(tournamentData);
          } catch (error) {
            console.error(error);
            setError(error.message);
          } finally {
            setLoading(false);
          }
        };

        fetchData();
      } else {
        setLoading(false);
      }
    }
  }, [router.isReady, router.query]);

  if (loading) return <div>Loading...</div>;
  if (error) return <div>{error}</div>;

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

            return (
              <div className='w-full h-full'>
                <p>{new Date(match.startTime*1000).toLocaleString()}</p>
                <div className="flex flex-col">
                  {topParticipant ? (

                    <div className="flex items-center">
                      <img
                        src={topParticipant.iconUrl || defaultPlayerIcon.blurDataURL}
                        alt={topParticipant.name}
                        className="w-8 mr-2"
                      />
                      <div className="flex-1">{topParticipant.name || 'Unknown'}</div>
                      <div>{topParticipant.resultText || (topParticipant.isWinner ? 'Win' : 'Loss')}</div>
                    </div>
                  ) : (
                    <div className="">No top participant</div>
                  )}

                  <div className="h-px w-full bg-yellow-500"></div>

                  {bottomParticipant ? (
                    <div className="flex items-center">
                      <img
                        src={bottomParticipant.iconUrl || defaultPlayerIcon.blurDataURL}
                        alt={bottomParticipant.name}
                        className="w-8 mr-2"
                      />
                      <div className="flex-1">{bottomParticipant.name || 'Unknown'}</div>
                      <div>{bottomParticipant.resultText || (bottomParticipant.isWinner ? 'Win' : 'Loss')}</div>
                    </div>
                  ) : (
                    <div className="">No bottom participant</div>
                  )}
                </div>
              </div>

            );
          }}
        // matchComponent={Match}
        />
      </div>
    </div>

  );
};

export default TournamentPage;