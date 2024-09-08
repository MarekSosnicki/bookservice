# Bookservice

This is a simple microservice based system using Rust for backend.
There are 3 main components of the system:

- `bookservice_repository` - a service with api that allows to add, update and query books
- `bookservice_reservations` - a service with api that allows to add users, reserve and unreserve books for users that
  additionally stores the history of reservations
- `bookservice_recommendations` - a service with api that periodically calculates recommendations for users and allows
  to retrieve them via API

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

To rebuild the system run

```bash
docker compose build
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

# APIs

With docker compose, all public APIs are available under port 80. Following endpoints are present:

- `GET /api/books` - list all books (ids and titles)
- `POST /api/book` - adds book to the repository
- `GET /api/book/{book_id}` - retrieve book details
- `GET /api/users` - lists all user ids
- `POST /api/user` - adds user
- `GET /api/user/{user_id}` - retrieve user details
- `POST /api/user/{user_id}/reservation/{book_id}` - reserves book for the user
- `DELETE /api/user/{user_id}/reservation/{book_id}` - unreserves book for the user
- `GET /api/user/{user_id}/history` - retrieve history of user reservations (only the unreserved ones)
- `GET /api/user/{user_id}/reservations` - retrieve active user reservations
- `GET /api/recommendations/{user_id}` - retrieve recommendations for user

The detail api spec can be found under:

- `/apispec/repository/v2` - spec of `/api/book` and `/api/books` endpoints
- `/apispec/reservations/v2` - spec of `/api/user` and `/api/users` endpoints
- `/apispec/recommendations/v2` - spec of `/api/recommendations`

# System details

## Bookservice repository

Bookservice repository is a simple microservice build based on Rust actix.
It uses postgres database to store book details.
You can set env variable `USE_IN_MEMORY_DB=true` to use the in memory database implementation (which will not persist
after reset).

## Bookservice reservations

Bookservice reservations is a simple microservice build based on Rust actix.
It uses postgres database to store user details, active reservations and history of user reservations.
You can set env variable `USE_IN_MEMORY_DB=true` to use the in memory database implementation (which will not persist
after reset).
The service calls `Bookservice repository` in order to validate that the book that user wants to reserve exists.

## Bookservice recommendations

Bookservice reservations is a microservice build based on Rust actix.
In parallel to the http api server, there is a periodic task running that calculates the recommendations for users.

Recommendations are calculated using two main structures:

- `CoefficientsStorage` - contains aggregated data of historical users reservations. Including:
    - `popularity_score` - describes by how many users each book was reserved
    - `author_to_books_sorted_by_popularity` - lists most popular books of each author
    - `author_match_score` - describes how often a pair of authors was reserved by the same user (how likely the two
      authors content match each other)
    - other useful lookup tables
- `RecommendationsEngine` - contains per user recommendations, recommendation are calculated using `CoefficientsStorage`
  and they
  are in following categories (up to `NO_OF_RECOMMENDATIONS` - default 5 books in each category):
    - `most_popular` - most popular books (based on `popularity_score`) that user has not reserved yet
    - `author_match` - most popular books of the authors that user already reserved books of but has not reserved yet
    - `new_author_match` - most popular books of the authors that user has never reserved books of, but they have the
      highest `author_match_score` based on historical user reservations

All data stored by this service is in memory, so after each restart everything is recalculated.

The recommendations are updated in ticks (default every 10s) in following pattern:

- every interval all newly added users have recommendations generated
- every 20 intervals, 10% of users are recalculated
- every 200 intervals (starting from the system startup), all books details are updated

# Remaining tasks

A list of ideas to improve the services:

- Adding metrics and dashboards to monitor the system
- Adding proper load testing
- Adding missing unit/integration tests coverage (especially for recommendations logic)
- Splitting recommendations service into 3 parts:
    - `CoefficientsStorage worker` - that would only update CoefficientsStorage and put it in Redis (or other db)
    - `RecommendationsEngine worker` - that would update recommendations for users when triggered and put it in Redis (
      or other db)
    - `api` - that would only server the recommendations
- Adding kafka, to replace the intervals logic in recommendations:
    - Update users only when they were updated (e.g. listen to `user` kafka topic)
    - Update books only when book was updated (e.g., listen to `book` kafka topic)
- Speedup building dockerfiles
- Add UI to the system
- Improve the CI (run integration tests and clippy)
