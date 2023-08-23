From rust:1.71 as builder
WORKDIR /usr/src/webeasy-rss
COPY . .
RUN cargo install --path .

From ubuntu:20.04
# RUN apt-get update && apt-get install -y extra-runtime-dependencies && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/webeasy-rss /usr/local/bin/webeasy-rss
RUN apt-get update && apt-get install -y openssl
CMD ["webeasy-rss", "-i", "0.0.0.0", "-p", "9000"]
EXPOSE 9000
