# Use the main rust docker image
FROM rust:latest AS builder

RUN apt-get update && apt-get -y upgrade
RUN apt-get install libssl-dev
RUN apt install -y pkg-config musl musl-dev musl-tools
RUN rustup target add x86_64-unknown-linux-musl

# copy app into docker image
COPY . /app

# Set the workdirectory
WORKDIR /app

# build the app
RUN cargo build --target x86_64-unknown-linux-musl --release --bin server

# We create the final Docker image “from scratch”
# FROM scratch
FROM alpine

# We copy our binary and the .env file over to
# the final image to keep it small
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/server /app/rust-web-dev
COPY --from=builder /app/.env /app/.env

WORKDIR /app

# start the application
CMD ["./rust-web-dev"]
