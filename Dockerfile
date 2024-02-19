FROM rust:1.76 as build

# Create a new empty shell project
RUN USER=root cargo new --bin fan-control
WORKDIR /fan-control

# Copy over manifests
COPY ./server/Cargo.lock ./Cargo.lock
COPY ./server/Cargo.toml ./Cargo.toml

# Install serialport crate dependencies
RUN apt-get update && apt-get install -y libudev-dev

# Build and cache the dependencies
RUN cargo build --release
RUN rm src/*.rs

# Copy over the source and build the application
COPY ./server/src ./src

# Build the application
RUN rm ./target/release/deps/fan_control*
RUN cargo build --release

# Final base image
# FROM debian:buster-slim
FROM rust:1.76-slim-bookworm

# Copy the build artifact from the build stage
COPY --from=build /fan-control/target/release/fan-control .

# Set the startup command to run the application
CMD ["./fan-control"]