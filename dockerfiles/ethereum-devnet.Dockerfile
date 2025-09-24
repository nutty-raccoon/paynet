FROM ghcr.io/foundry-rs/foundry:latest

USER root
RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*