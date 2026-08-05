FROM mcr.microsoft.com/devcontainers/rust:1-1-bookworm

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        build-essential \
        clang \
        cmake \
        lld \
        make \
        pkg-config \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*
