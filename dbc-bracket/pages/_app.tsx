import Footer from '@/components/footer';
import '../styles/global.css';
import Navbar from '@/components/NavBar';
import { AppProps } from 'next/app';

function MyApp({ Component, pageProps }: AppProps) {
    return (
        <div className='w-full h-screen flex flex-col'>
            <div className='w-full h-[10vh]'>
            <Navbar />
            </div>
            <div className='w-full h-[80vh] md:h-[85vh]'>
            <Component {...pageProps} />
            </div>
            <div className='w-full h-[10vh] md:h-[5vh]'>
            <Footer />
            </div>
        </div>
    );
}

export default MyApp;
