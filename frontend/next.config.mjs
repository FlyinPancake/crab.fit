// @ts-check

import { dirname, resolve } from "path"
import { fileURLToPath } from "url"

const __dirname = dirname(fileURLToPath(import.meta.url))

/** @type {import('next').NextConfig} */
const nextConfig = {
  output: "standalone",
  webpack: config => {
    config.resolve.alias = {
      ...config.resolve.alias,
      // "@js-temporal/polyfill": path.resolve(
      //   __dirname,
      //   "node_modules/@js-temporal/polyfill/dist/index.esm.js",
      // ),
      jsbi: resolve(__dirname, "node_modules/jsbi/dist/jsbi.mjs"),
    }
    return config
  },
}

export default nextConfig
