"use client";
import { usePathname } from 'next/navigation';
import { ReactNode, useEffect, useState } from 'react';
import Sidebar from '../components/sidebar';
import ErrorBoundary from '../components/error_boundary';

interface GuildPage {
  children: ReactNode;
}

const GuildPage: React.FC<GuildPage> = ({ children }) => {
  const router = usePathname() as string;
  const guildId = router.split("/").pop();
  const [items, setItems] = useState<string[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (guildId) {
      const fetchData = async () => {
        try {
          const response = await fetch(`/api/${guildId}`, {
            method: 'GET'
          });
          if (!response.ok) {
            throw new Error('Network response was not ok');
          }

          const tournaments = await response.json();
          setItems(tournaments.map((tournament: { name: string }) => tournament.name));
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
  }, [guildId]);

  if (loading) return <div>Loading...</div>;
  if (error) return <div>{error}</div>;

  return (
    <div>
        <Sidebar items={items} />
        <main>{children}</main>
    </div>
  );
};

export default GuildPage;
