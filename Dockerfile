FROM rust:1.66 as restore
WORKDIR /usr/src
COPY Cargo.* .
COPY common/Cargo.* ./common/
COPY api/Cargo.* ./api/
COPY web/Cargo.* ./web/
RUN mkdir ./common/src && mkdir ./api/src && mkdir ./web/src
RUN echo "fn main() {}" > ./common/src/lib.rs && echo "fn main() {}" > ./api/src/main.rs && echo "fn main() {}" > ./web/src/lib.rs
RUN cargo build

FROM restore as build
WORKDIR /usr/src
COPY common/ ./common/
COPY api/ ./api/
ENV SQLX_OFFLINE=true
RUN cargo install --path ./api

FROM restore as web-build
WORKDIR /usr/src
RUN apt-get update && apt-get install -y nodejs npm
RUN echo "API_BASE_PATH=http://localhost:3000/api/\nOAUTH_CLIENT_ID=709154627100-fbcvr0njtbah2jfgv5bghnt7t39r28k9.apps.googleusercontent.com" > .env
COPY common/ ./common/
WORKDIR /usr/src/web
COPY web/package*.json .
RUN npm ci
COPY web/ .
RUN npm run build

FROM nginx:latest
COPY api-entrypoint.sh /docker-entrypoint.d
COPY default.conf /etc/nginx/conf.d/
COPY --from=build /usr/local/cargo/bin/api /usr/local/bin/api
COPY --from=web-build /usr/src/web/dist /data/www