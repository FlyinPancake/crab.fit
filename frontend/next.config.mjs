// @ts-check

/** @type {import('next').NextConfig} */
const nextConfig = {
  output: "standalone",
  turbopack: {
    resolveAlias: {
      jsbi: "./node_modules/jsbi/dist/jsbi.mjs",
    },
  },
  rewrites: async () => [
    {
      source: "/api/:path*",
      destination: "http://127.0.0.1:3000/:path*",
    },
  ],
};

export default nextConfig;
