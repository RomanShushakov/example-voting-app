FROM rust:1.74 as builder
RUN mkdir -p /app/
WORKDIR /app/
COPY . /app/
RUN cargo install --path .

FROM debian:bookworm-slim
RUN mkdir -p /app/
WORKDIR /app/
RUN apt update && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/result /usr/local/bin/result
EXPOSE 8081
