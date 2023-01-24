FROM rust:1.66 as builder
WORKDIR /usr/src/
COPY . .
ENV SQLX_OFFLINE=true
RUN cargo install --path ./api

FROM ubuntu:latest
COPY --from=builder /usr/local/cargo/bin/api /usr/local/bin/api
EXPOSE 3000
CMD ["api"]