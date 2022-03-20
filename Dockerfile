FROM rust:1.59.0 as builder
WORKDIR /usr/src/app

COPY . .

RUN cargo build --release

# Bundle stage
FROM debian:bullseye-slim
COPY --from=builder /usr/src/app/target/release/hookrunner hookrunner
RUN apt-get update && apt-get install ca-certificates git -y && rm -rf /var/lib/apt/lists/*
USER 1000

ENTRYPOINT ["/hookrunner"]
CMD ["run"]
