"use client";
import { SingleEliminationBracket, Match, SVGViewer } from '@g-loot/react-tournament-brackets';
import { MatchType, ParticipantType } from '../../../db/models';
import { ReactNode, useEffect, useState } from 'react';
import { useRouter } from 'next/router';
import { useWindowSize } from '@uidotdev/usehooks';
import brawlStarsBackground from '../../../../images/assets/battle_log_bg.png';
import defaultPlayerIcon from '../../../../images/assets/28000000.png';

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
    <div>
      <div
        style={{
          display: 'flex',
          justifyContent: 'center',
          alignItems: 'center',
          minHeight: '100vh',
          padding: '20px',
          backgroundColor: '#000',
        }}
      >
        <SingleEliminationBracket
          matches={matches}
          options={{
            style: {
              roundHeader: { backgroundColor: '#FFD700' },
              connectorColor: '#FFD700',
            },
          }}
          svgWrapper={({ children, ...props }) => (
            <SVGViewer
              background="#000"
              SVGBackground="#000"
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
              <div style={{ display: 'flex', flexDirection: 'column', padding: '10px' }}>
                {topParticipant ? (
                  <div style={{ display: 'flex', alignItems: 'center', marginBottom: '10px' }}>
                    <img src={topParticipant.iconUrl || defaultPlayerIcon.blurDataURL} alt={topParticipant.name} style={{ width: '30px', marginRight: '10px' }} />
                    <div style={{ flex: 1, color: '#FFFFFF' }}>{topParticipant.name || 'Unknown'}</div>
                    <div>{topParticipant.resultText || (topParticipant.isWinner ? 'Win' : 'Loss')}</div>
                  </div>
                ) : (
                  <div style={{ marginBottom: '10px' }}>No top participant</div>
                )}
  
                <div style={{ height: '2px', width: '100%', backgroundColor: '#FFD700' }}></div>
  
                {bottomParticipant ? (
                  <div style={{ display: 'flex', alignItems: 'center', marginTop: '10px' }}>
                    <img src={bottomParticipant.iconUrl || defaultPlayerIcon.blurDataURL} alt={bottomParticipant.name} style={{ width: '30px', marginRight: '10px' }} />
                    <div style={{ flex: 1, color: '#FFFFFF' }}>{bottomParticipant.name || 'Unknown'}</div>
                    <div>{bottomParticipant.resultText || (bottomParticipant.isWinner ? 'Win' : 'Loss')}</div>
                  </div>
                ) : (
                  <div style={{ marginTop: '10px' }}>No bottom participant</div>
                )}
              </div>
            );
          }}
        />
      </div>
    </div>
  );
};

export default TournamentPage;