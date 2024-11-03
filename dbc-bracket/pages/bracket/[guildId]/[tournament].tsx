"use client";
import { SingleEliminationBracket, SVGViewer, createTheme, Match } from '@g-loot/react-tournament-brackets';
import { MatchType, ParticipantType } from '@/db/models';
import { ReactNode, useEffect, useState } from 'react';
import { useRouter } from 'next/router';
import { useWindowSize } from '@uidotdev/usehooks';
import brawlStarsBackground from '@/../images/assets/battle_log_bg.png';
import defaultPlayerIcon from '@/../images/assets/28000000.png';
import BracketService from '@/pages/services/bracket';

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
          const [response, error] = await BracketService.getBracket(guildId as string, tournament as string);
          setLoading(false);
          if (error) {
            setError(error.message);
            return;
          }
          setMatches(response);
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
                      <img
                        src={topParticipant.iconUrl || defaultPlayerIcon.blurDataURL}
                        alt={topParticipant.name}
                        className="w-8 mr-2"
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
                      <img
                        src={bottomParticipant.iconUrl || defaultPlayerIcon.blurDataURL}
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

  );
};

const TBD: React.FC = () => {
  return (
    <div className="flex items-center">
      <img
        src="https://cdn.brawlify.com/profile-icons/regular/28000000.png"
        alt="TBD"
        className="w-8 mr-2"
      />
      <div className="flex-1">TBD</div>
      <div>TBD</div>
    </div>
  )
}

export default TournamentPage;