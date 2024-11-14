// next.config.js

import webpack from 'webpack';
/** @type {import('next').NextConfig} */
const nextConfig = {
    output: "standalone",
    webpack(config) {
        config.plugins.push(new webpack.IgnorePlugin({ resourceRegExp: /^pg-native$/ }));
        
        config.resolve.fallback = {
            ...config.resolve.fallback,
            fs: false,
            dns: false,
            net: false,
            tls: false,
        };
        
        return config;
    },
};

export default nextConfig;
