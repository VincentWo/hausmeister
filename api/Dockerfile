FROM rustlang/rust:nightly-alpine3.15 as chef
WORKDIR /app
RUN apk update && apk add g++
ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL sparse
RUN cargo install cargo-chef

FROM chef as planner
COPY . .
# Compute a lock-like file for our project
RUN cargo chef prepare --recipe-path recipe.json


FROM chef as Builder
WORKDIR /app
RUN apk update && apk add musl-dev openssl pkgconfig openssl-dev perl make

COPY --from=planner /app/recipe.json recipe.json
# Build our project dependencies, not our application!
RUN cargo chef cook --release --recipe-path recipe.json
# Up to this point, if our dependency tree stays the same,
# all layers should be cached.
COPY . .

ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL sparse
ENV SQLX_OFFLINE true
RUN cargo build --release --bin hausmeister

FROM dumb-init as Runner
WORKDIR /app
COPY --from=Builder /app/target/release/hausmeister /app/hausmeister

ENV HM_APP__LISTEN_ON="0.0.0.0"
ENTRYPOINT ["./hausmeister"]

