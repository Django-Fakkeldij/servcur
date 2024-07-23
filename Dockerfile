FROM clux/muslrust:stable AS rust_builder
WORKDIR /app

COPY Cargo.toml .
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release --target x86_64-unknown-linux-musl

COPY src src
RUN touch src/main.rs
RUN cargo build --release --target x86_64-unknown-linux-musl

RUN strip /app/target/x86_64-unknown-linux-musl/release/servcur

FROM node:21 as node_builder

WORKDIR /app

COPY public/servcur .

RUN npm install
RUN npm run build


FROM alpine:latest as release
RUN apk update && apk add git && apk add bash
WORKDIR /app
COPY --from=rust_builder /app/target/x86_64-unknown-linux-musl/release/servcur .
COPY --from=node_builder /app/build ./public/servcur/build

CMD ["./servcur"]