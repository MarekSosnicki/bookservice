# Based on: https://github.com/actix/examples/blob/master/docker/Dockerfile
#################
## build stage ##
#################
FROM rust:1-slim-bookworm AS builder
WORKDIR /code/service
COPY . .
RUN cargo build --bin bookservice_reservations --release

###############
## run stage ##
###############
FROM bitnami/minideb:bookworm
WORKDIR /app

# copy server binary from build stage
COPY --from=builder /code/service/target/release/bookservice_reservations bookservice_reservations

USER 1001
EXPOSE 8080

# run server
CMD [ "/app/bookservice_reservations" ]