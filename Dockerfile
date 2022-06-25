FROM rust:latest AS builder

RUN rustup target add x86_64-unknown-linux-musl
RUN apt update && apt install -y musl-tools musl-dev
RUN update-ca-certificates

WORKDIR /data

COPY ./ .

RUN cargo build --target x86_64-unknown-linux-musl --release


FROM scratch

WORKDIR /data

COPY --from=builder /data/target/x86_64-unknown-linux-musl/release/simple_pixiv ./

EXPOSE 8080

ENV PORT=8080

CMD ["/data/simple_pixiv"]