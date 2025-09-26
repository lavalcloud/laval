FROM node:20-alpine AS builder
WORKDIR /app

ENV PNPM_HOME=/root/.local/share/pnpm
ENV PATH="$PNPM_HOME:$PATH"
RUN corepack enable

COPY package.json pnpm-lock.yaml pnpm-workspace.yaml ./
COPY packages ./packages
COPY thirdparty ./thirdparty
RUN pnpm install --frozen-lockfile
RUN pnpm --filter frontend run build

FROM caddy:alpine AS runtime

COPY --from=builder /app/packages/frontend/dist /usr/share/caddy

COPY Caddyfile /etc/caddy/Caddyfile

EXPOSE 80
CMD ["caddy", "run", "--config", "/etc/caddy/Caddyfile", "--adapter", "caddyfile"]