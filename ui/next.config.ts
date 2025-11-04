import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  output: 'standalone',
  // Disable telemetry
  reactStrictMode: true,
};

export default nextConfig;
