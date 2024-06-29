FROM rust:alpine as builder
WORKDIR /usr/src/myapp
COPY . .
RUN apk add --no-cache musl-dev
RUN cargo install --path sonarr-script

FROM alpine:latest
RUN apk add --no-cache ffmpeg
COPY --from=builder /usr/local/cargo/bin/sonarr-script /usr/local/bin/sonarr-script
CMD ["sonarr-script"]
LABEL org.opencontainers.image.source https://github.com/pbar1/arr-scripts
