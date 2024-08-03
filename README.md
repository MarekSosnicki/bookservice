# Bookservice

This is a simple microservice based system using Rust for backend.
There are 3 main components of the system:

- `bookservice_repository` - a service with api that allows to add, update and query books
- `bookservice_reservations` - a service with api that allows to add users, reserve and unreserve books for users that
  additionally stores the history of reservations
- `bookservice_recommendations` - a service with api that periodically calculates recommendations for users and allows
  to retrieve them via API

The spec of each API can be found after launching the app under `/apispec/v2` path.

# Requirements

For running the system Docker setup should be enough.

For building and testing:

- Rust v1.80 or newer
- Docker setup to handle testcontainers (On windows this requires Clang, Cmake and NASM )

# Starting the system

To start the system run:

```bash
docker compose up
```

# Testing

To run unit tests:

```bash
cargo test
```

To run system e2e tests, first start the docker compose (with `docker compose up`) and then run

```
cargo test --features system_tests
```

> Note: system tests leave some artifacts in the system (e.g. create some test books that are not removed later)
> as there is no api to remove items yet

# API

# TODO List

- bookservice_admin_api
    - rename?
    - add metrics
    - add unit tests
- bookservice_user_api
    - rename?
    - add api
    - add postgres
    - add metrics
    - add unit tests
    - add to docker compose
- bookservice recommendations
    - add implmentation
    - add postgres?
- Other
    - Add readme
    - add api documentation (paperclip?)
    - add nginx for a single endpoint?
    - Add integration test for the backend
    - add health checks for docker
    - 
  