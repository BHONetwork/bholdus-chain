FROM rust:1.55.0 as builder

RUN apt-get update && apt-get install -y git libclang-dev

WORKDIR /bholdus

COPY . .

RUN ./scripts/init.sh

RUN cargo build --release --features with-bholdus-runtime

RUN mv ./target/release/bholdus /usr/local/bin

FROM rust:1.55.0 as runner

COPY --from=builder /usr/local/bin/bholdus /usr/local/bin/bholdus
