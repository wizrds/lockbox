# syntax=docker/dockerfile:1
ARG RUST_VERSION=1.95.0
FROM rust:${RUST_VERSION}-alpine AS build

ARG RUST_VERSION=1.95.0

RUN apk add --no-cache \
    build-base \
    pkgconfig \
    curl \
    cmake \
    openssl-dev \
    openssl-libs-static \
    zlib-dev \
    zlib-static \
    musl-dev

WORKDIR /workspace

COPY . .

# Build statically against musl
RUN cargo build --release --features mimalloc

FROM alpine:3.21 AS runtime

RUN apk add --no-cache \
    curl \
    ca-certificates \
    shadow \
    sudo

RUN useradd -u 1000 -G wheel -U -m -s /bin/sh lockbox \
    && echo "lockbox ALL=(ALL) NOPASSWD: /bin/chown" >> /etc/sudoers

RUN mkdir -p /srv/lockbox && chown lockbox:lockbox /srv/lockbox
WORKDIR /srv/lockbox

ENV PATH="/usr/local/bin:$PATH"
RUN echo 'export PATH=/usr/local/bin:$PATH' >> /home/lockbox/.profile

COPY --from=build --chown=lockbox:lockbox /workspace/target/release/lockbox /usr/local/bin/lockbox

USER lockbox
ENTRYPOINT ["lockbox"]
CMD ["help"]
