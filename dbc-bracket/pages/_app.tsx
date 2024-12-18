import Footer from '@/components/footer';
import '../styles/global.css';
import Navbar from '@/components/NavBar';
import { AppProps } from 'next/app';

function MyApp({ Component, pageProps }: AppProps) {
    return (
        <div className='w-full h-screen flex flex-col'>
            <Navbar />
            <Component {...pageProps} />
            <Footer />
        </div>
    );
}

export default MyApp;
