FROM rust:latest AS builder
WORKDIR /app

COPY Cargo.toml .
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release

COPY src src
RUN touch src/main.rs
RUN cargo build --release

RUN strip target/release/servcur

FROM gcr.io/distroless/cc-debian12 as release
WORKDIR /app
COPY --from=builder /app/target/release/servcur .
COPY public public

EXPOSE 3000

CMD ["./servcur"]