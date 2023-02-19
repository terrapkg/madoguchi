FROM rust:alpine as build

RUN USER=root cargo new --bin madoguchi
WORKDIR /madoguchi

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

RUN cargo build --release
RUN rm src/*.rs

COPY ./src ./src
COPY ./migrations ./migrations
COPY ./Rocket.toml ./Rocket.toml

RUN rm ./target/release/deps/madoguchi*
RUN cargo build --release

FROM rust:alpine

COPY --from=build /madoguchi/target/release/madoguchi .

CMD ["./madoguchi"]
