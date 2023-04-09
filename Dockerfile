FROM clux/muslrust:stable AS builder

RUN rustup target add x86_64-unknown-linux-musl
RUN apt update && apt install -y musl-tools musl-dev pkg-config libssl-dev libudev-dev
RUN update-ca-certificates

# Create appuser
ENV USER=chessws
ENV UID=10001

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"


WORKDIR /ws

COPY ./ .

RUN cargo build --target x86_64-unknown-linux-musl --release

####################################################################################################
## Final image
####################################################################################################
FROM alpine:latest

# Import from builder.
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

WORKDIR /ws

# Copy our build
COPY --from=builder /ws/target/x86_64-unknown-linux-musl/release/ws ./

# Use an unprivileged user.
USER chessws:chessws

CMD ["/ws/ws"]