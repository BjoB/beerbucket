FROM rust:latest

LABEL maintainer="Björn Barschtipan"

WORKDIR /beerbucket/
COPY . .

RUN rustc --version
RUN cargo build --release
RUN cargo install --path .

CMD ["beerbucket-server"]
