FROM rust:alpine as builder

WORKDIR /usr/src/

RUN apk update && apk add --no-cache \
    git \
    ca-certificates \
    tzdata \
    gcc \
    musl-dev \
    && update-ca-certificates

RUN adduser -D -g '' appuser

Run USER=root cargo new memsther
WORKDIR /usr/src/memsther

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations

RUN cargo build --release

FROM scratch

COPY --from=builder /usr/share/zoneinfo /usr/share/zoneinfo
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY --from=builder /etc/passwd /etc/passwd

COPY --from=builder /usr/src/memsther/target/release/memsther /target/memsther
COPY --from=builder /usr/src/memsther/migrations /target/migrations

WORKDIR /target

USER appuser

ENTRYPOINT ["/target/memsther"]

