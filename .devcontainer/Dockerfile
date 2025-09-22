FROM mcr.microsoft.com/devcontainers/rust:1

RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    && apt-get -y install --no-install-recommends \
      just cmake

COPY --from=ghcr.io/astral-sh/uv:latest /uv /uvx /bin/
