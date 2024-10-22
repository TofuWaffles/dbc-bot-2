"use client";

import { usePathname } from 'next/navigation';
import { SingleEliminationBracket, Match, SVGViewer, MatchType } from '@g-loot/react-tournament-brackets';
import { ReactNode, useEffect, useState } from 'react';

interface TournamentPage {
  children: ReactNode;
}

const TournamentPage: React.FC<TournamentPage> = ({ children }) => {
  const router = usePathname() as string;
  const tournamentName = router.split("/").pop();
  const [matches, setMatches] = useState<MatchType[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (tournamentName) {
      const fetchData = async () => {
        try {
          const response = await fetch(`/pages/api/tournaments/${tournamentName}`);
          if (!response.ok) {
            throw new Error('Network response was not ok');
          }
          const tournamentData = await response.json();
          setMatches(tournamentData);
        } catch (error) {
          console.error(error);
          setError("Failed to load match data");
        } finally {
          setLoading(false);
        }
      };

      fetchData();
    } else {
      setLoading(false);
    }
  }, [tournamentName]);

  if (loading) return <div>Loading...</div>;
  if (error) return <div>{error}</div>;

  return (
    <SingleEliminationBracket
      matches={matches}
      matchComponent={Match}
      svgWrapper={({ children, ...props }) => (
        <SVGViewer width={500} height={500} {...props}>
          {children}
        </SVGViewer>
      )}
    />
  );
};

export default TournamentPage;