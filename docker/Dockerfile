# AIO Dockerfile

ARG RUST_VERSION=1.95.0-stable
ARG NODE_VERSION=20

FROM clux/muslrust:${RUST_VERSION} AS chef

USER root
RUN cargo install --locked cargo-chef
WORKDIR /app

FROM chef AS planner
COPY api/ .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
ARG ADAPTOR
ENV ADAPTOR=${ADAPTOR}

COPY --from=planner /app/recipe.json recipe.json
# Notice that we are specifying the --target flag!
RUN cargo chef cook --release --target $(uname -m)-unknown-linux-musl --features ${ADAPTOR} --recipe-path recipe.json
COPY api/ .
RUN cargo build --release --target $(uname -m)-unknown-linux-musl --features ${ADAPTOR} && \
    cp target/$(uname -m)-unknown-linux-musl/release/crabfit-api /usr/local/bin/crabfit-api

# Frontend

FROM node:${NODE_VERSION}-alpine AS node-core
WORKDIR /app

FROM node-core AS node-dependencies

COPY frontend/package.json frontend/yarn.lock ./
RUN corepack enable && yarn install --frozen-lockfile

FROM node-core AS node-build

# TODO: find replacement
ARG NEXT_PUBLIC_API_URL=http://localhost:8080/api/
# ARG NEXT_PUBLIC_GOOGLE_CLIENT_ID
# ARG NEXT_PUBLIC_GOOGLE_API_KEY
ENV NEXT_PUBLIC_API_URL=$NEXT_PUBLIC_API_URL
# ENV NEXT_PUBLIC_GOOGLE_CLIENT_ID=$NEXT_PUBLIC_GOOGLE_CLIENT_ID
# ENV NEXT_PUBLIC_GOOGLE_API_KEY=$NEXT_PUBLIC_GOOGLE_API_KEY

COPY --from=node-dependencies /app/node_modules ./node_modules
COPY frontend/ .

ENV NODE_ENV=production API_URL=http://127.0.0.1:3000
RUN corepack enable && yarn build

# Runtime: bundle API + frontend behind s6-overlay

FROM node:${NODE_VERSION}-alpine AS runtime

ARG S6_OVERLAY_VERSION=3.2.0.2

RUN apk add --no-cache ca-certificates tzdata xz \
    && ARCH="$(uname -m)" \
    && wget -qO /tmp/s6-noarch.tar.xz "https://github.com/just-containers/s6-overlay/releases/download/v${S6_OVERLAY_VERSION}/s6-overlay-noarch.tar.xz" \
    && wget -qO /tmp/s6-arch.tar.xz "https://github.com/just-containers/s6-overlay/releases/download/v${S6_OVERLAY_VERSION}/s6-overlay-${ARCH}.tar.xz" \
    && tar -C / -Jxpf /tmp/s6-noarch.tar.xz \
    && tar -C / -Jxpf /tmp/s6-arch.tar.xz \
    && rm /tmp/s6-noarch.tar.xz /tmp/s6-arch.tar.xz

WORKDIR /app

# API binary (musl static)
COPY --from=builder /usr/local/bin/crabfit-api /usr/local/bin/crabfit-api

# Next.js standalone bundle
COPY --from=node-build /app/.next/standalone /app/frontend/
COPY --from=node-build /app/.next/static /app/frontend/.next/static
COPY --from=node-build /app/public /app/frontend/public

# s6-rc service definitions
COPY docker/s6-rc.d /etc/s6-overlay/s6-rc.d

ENV NODE_ENV=production \
    API_URL=http://127.0.0.1:3000 \
    HOSTNAME=0.0.0.0 \
    PORT=8080

# Frontend port (API stays internal on 3000)
EXPOSE 8080

ENTRYPOINT ["/init"]
