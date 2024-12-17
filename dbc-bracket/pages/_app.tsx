import Footer from '@/components/footer';
import '../styles/global.css'; 
import Navbar from '@/components/NavBar';
import { AppProps } from 'next/app';

function MyApp({ Component, pageProps }: AppProps) {
    return <>
         <Navbar />
         <Component {...pageProps} />
         <Footer />
    </>;
}

export default MyApp;
