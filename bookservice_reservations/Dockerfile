# Based on: https://github.com/actix/examples/blob/master/docker/Dockerfile
#################
## build stage ##
#################
FROM rust:slim as builder
RUN apt-get update && apt-get install -y \
    build-essential checkinstall zlib1g-dev pkg-config libssl-dev -y
WORKDIR /code/service
COPY . .
RUN cargo build --bin bookservice_reservations --release

###############
## run stage ##
###############
FROM bitnami/minideb:bookworm
# This may fix open ssl issue
RUN apt-get update && apt-get install -y \
    build-essential checkinstall zlib1g-dev pkg-config libssl-dev -y

WORKDIR /app

# copy server binary from build stage
COPY --from=builder /code/service/target/release/bookservice_reservations bookservice_reservations

USER 1001
EXPOSE 8080

# run server
CMD [ "/app/bookservice_reservations" ]