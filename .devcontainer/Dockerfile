FROM mcr.microsoft.com/devcontainers/rust:1-1-bookworm

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        build-essential \
        clang \
        cmake \
        lld \
        make \
        pkg-config \
        curl \
        git \
        ca-certificates \
    && curl -s https://raw.githubusercontent.com/nektos/act/master/install.sh \
        | bash -s -- -b /usr/local/bin \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*
