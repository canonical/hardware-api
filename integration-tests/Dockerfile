FROM ubuntu:24.04

# Install Rust and required packages
RUN apt-get update && DEBIAN_FRONTEND=noninteractive apt-get install -y \
    curl \
    build-essential \
    pkgconf \
    libssl-dev \
    lsb-release \
    kmod \
    python3 \
    python3-pip \
    rustup

RUN rustup default stable

WORKDIR /app

COPY Cargo.toml .
COPY src src/

CMD ["cargo", "test", "--test", "integration", "--", "--nocapture"]
