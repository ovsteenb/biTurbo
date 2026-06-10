/** @type {import('next').NextConfig} */

const repo = process.env.GITHUB_REPOSITORY?.split("/")[1] || "biturbo-landing";
const isGithubActions = !!process.env.GITHUB_ACTIONS;
const basePath = process.env.NEXT_PUBLIC_BASE_PATH ?? (isGithubActions ? `/${repo}` : "");

const nextConfig = {
  reactStrictMode: true,
  output: "export",
  images: {
    unoptimized: true,
    remotePatterns: [
      { protocol: "https", hostname: "**" },
    ],
  },
  basePath: basePath || undefined,
  assetPrefix: basePath || undefined,
  trailingSlash: true,
};

export default nextConfig;

