import Header from '@/components/header';
import '../styles/global.css';
import Navbar from '@/components/NavBar';
import { AppProps } from 'next/app';

function MyApp({ Component, pageProps }: AppProps) {
    return (
        <div className='w-full h-screen flex flex-col'>
            <div className='w-full h-[10vh]'>
            <Navbar />
            <Header />
            </div>
            <div className='w-full'>
            <Component {...pageProps} />
            </div>
        </div>
    );
}

export default MyApp;
