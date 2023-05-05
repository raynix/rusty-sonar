FROM rust:1.69 as builder
WORKDIR /usr/src/myapp
COPY . .
RUN cargo build --release

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/myapp/target/release/rusty-sonar /usr/local/bin/rusty-sonar
CMD ["rusty-sonar"]
