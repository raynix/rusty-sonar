FROM rust:1.89 as builder
WORKDIR /usr/src/myapp
COPY . .
RUN cargo build --release

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y ca-certificates openssl && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/myapp/target/release/rusty-sonar /usr/local/bin/rusty-sonar
CMD ["rusty-sonar"]
