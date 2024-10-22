/** @type {import('next').NextConfig} */
const nextConfig = {
    webpack(config) {
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
