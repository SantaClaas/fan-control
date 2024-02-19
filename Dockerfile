# Build the client SPA
FROM node:21 as build-client

WORKDIR /client

# Copy over manifests
COPY ./client/package.json ./package.json
COPY ./client/package-lock.json ./package-lock.json

# Install dependencies
RUN npm ci

# Copy over the source to build the application
COPY ./client ./

# Build and cache
RUN npm run build

# Build the server containing the API and hosting the client
FROM rust:1.76 as build-server

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

# Copy over the source to build the application
COPY ./server/src ./src

# Build the application
RUN rm ./target/release/deps/fan_control*
RUN cargo build --release

# Final base image
# FROM debian:buster-slim
FROM rust:1.76-slim-bookworm as final

# Copy the build artifacts from the build stage
COPY --from=build-server /fan-control/target/release/fan-control .
# The ./client directory is where the server looks for when client static files are requested
COPY --from=build-client /client/dist ./client

# Set the startup command to run the application
CMD ["./fan-control"]