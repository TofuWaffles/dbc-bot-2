import { Html, Head, Main, NextScript } from 'next/document';

export default function Document() {
  return (
    <Html lang="en">
      <Head>
        <meta charSet="UTF-8" />
        <meta property="og:site_name" content="Discord Brawl Cup"/>
        <meta property='og:description' content="Discord Brawl Cup is /r/BrawlStars Discord Server's in-house competition where players face each other in a 1v1 bracket-style tournament to win in-game prizes!" />
        <meta property="og:type" content="website" />
        <meta name="theme-color" content="#000000" />
      </Head>
      <body>
        <Main />
        <NextScript />
      </body>
    </Html>
  );
}
