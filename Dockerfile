FROM rust:latest as build

RUN USER=root cargo new --bin madoguchi
WORKDIR /madoguchi

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

RUN USER=root cargo new --bin xtask
COPY ./xtask/Cargo.toml ./xtask/Cargo.toml

RUN cargo build --release
RUN rm src/*.rs xtask/src/*.rs

COPY ./src ./src
COPY ./xtask/src ./xtask/src
COPY ./migrations ./migrations
COPY ./Rocket.toml.example ./Rocket.toml
COPY ./sqlx-data.json ./sqlx-data.json

RUN rm ./target/release/deps/madoguchi*
RUN cargo build --release

FROM rust:latest

COPY --from=build /madoguchi/target/release/madoguchi .

CMD ["./madoguchi"]
