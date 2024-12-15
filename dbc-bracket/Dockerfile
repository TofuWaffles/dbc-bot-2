# syntax=docker.io/docker/dockerfile:1


FROM oven/bun:alpine AS base

WORKDIR /app

FROM base AS deps
COPY package.json bun.lockb ./

RUN bun install --frozen-lockfile

FROM base AS builder
WORKDIR /app
COPY --from=deps /app/bun.lockb ./bun.lockb
COPY --from=deps /app/node_modules ./node_modules
COPY . .

# Set environment variable placeholder for database connection
ENV DATABASE_URL="postgresql://user:password@localhost:5432/PLACEHOLDER"

RUN bun run build

# Production image, copy all the files and run the app
FROM base AS runner
WORKDIR /app

ENV NODE_ENV=production
COPY --from=builder /app/public ./public

# Set permissions for cache directories
RUN mkdir .next && chown -R 1001:1001 .next

# Copy output files
COPY --from=builder --chown=1001:1001 /app/.next/standalone ./
COPY --from=builder --chown=1001:1001 /app/.next/static ./.next/static

# Use a non-root user
USER 1001

EXPOSE 3000

ENV PORT=3000
ENV HOSTNAME="0.0.0.0"

# Start the server with Bun
CMD ["bun", "run", "server.js"]
