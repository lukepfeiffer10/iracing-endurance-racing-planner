FROM rust:1.66 as pre-build
WORKDIR /usr/src
COPY Cargo.* .
COPY common/Cargo.* ./common/
COPY api/Cargo.* ./api/
COPY web/Cargo.* ./web/
RUN mkdir ./common/src && mkdir ./api/src && mkdir ./web/src
RUN echo "fn main() {}" > ./common/src/lib.rs && echo "fn main() {}" > ./api/src/main.rs && echo "fn main() {}" > ./web/src/lib.rs
RUN cargo build

FROM pre-build as build
WORKDIR /usr/src
COPY common/src/ ./common/src/
COPY api/sqlx-data.json ./api/
COPY api/src/ ./api/src/
COPY web/src/ ./web/src/
ENV SQLX_OFFLINE=true
RUN cargo install --path ./api

FROM nginx:latest
COPY api-entrypoint.sh /docker-entrypoint.d
COPY default.conf /etc/nginx/conf.d/
COPY --from=build /usr/local/cargo/bin/api /usr/local/bin/api