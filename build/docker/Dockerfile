# syntax=docker/dockerfile:1
ARG RUST_VERSION=1.94.0
FROM rust:${RUST_VERSION}-slim-bookworm AS build

ARG RUST_VERSION=1.94.0

RUN DEBIAN_FRONTEND=noninteractive apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    curl \
    cmake \
    libssl-dev \
    libz-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /workspace

COPY . .

RUN cargo build --release --features mimalloc

FROM debian:bookworm-slim AS runtime

RUN useradd -u 1000 -G sudo -U -m -s /bin/bash lockbox \
    && echo "lockbox ALL=(ALL) NOPASSWD: /bin/chown" >> /etc/sudoers
RUN mkdir -p /srv/lockbox && chown lockbox:lockbox /srv/lockbox
WORKDIR /srv/lockbox

ENV PATH="/usr/local/bin:$PATH"
RUN echo 'export PATH=/usr/local/bin:$PATH' >> /home/lockbox/.profile

COPY --from=build --chown=lockbox:lockbox /workspace/target/release/lockbox /usr/local/bin/lockbox

USER lockbox
ENTRYPOINT ["lockbox"]
CMD ["help"]