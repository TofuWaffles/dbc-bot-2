import SSEClient from '@/components/SSEClient';
import { useRouter } from 'next/router';
import { usePathname } from 'next/navigation';

const BracketHome = () => {
    const router = useRouter();
    const { tournamentId } = router.query;
    const pathname = usePathname();

    if (!tournamentId) {
        return <div>Invalid tournament ID</div>;
    }

    return <SSEClient path={`/api${pathname}`} />;
};

export default BracketHome;