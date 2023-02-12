FROM rust:latest AS builder

RUN rustup target add x86_64-unknown-linux-musl

RUN apt update && apt install -y \
    musl-tools \
    musl-dev \
    pkg-config

RUN update-ca-certificates

ENV USER=memsther
ENV UID=10001

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"

WORKDIR /memsther

COPY ./ .

RUN cargo build --target x86_64-unknown-linux-musl --release


FROM scratch

COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

WORKDIR /memsther
COPY --from=builder /memsther/target/x86_64-unknown-linux-musl/release/memsther ./memsther
COPY --from=builder /memsther/migrations ./migrations

USER memsther:memsther

ENTRYPOINT ["/memsther/memsther"]
