# Build stage
FROM oven/bun:latest AS build
WORKDIR /app
COPY . .
RUN bun install
RUN bun run build

# Production stage
FROM oven/bun:alpine AS production
WORKDIR /app
COPY --from=build /app ./
EXPOSE 3000
CMD ["bun", "start"]
