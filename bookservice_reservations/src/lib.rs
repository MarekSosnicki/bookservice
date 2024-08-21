pub mod api;

#[cfg(any(feature = "client", test))]
pub mod client;

#[cfg(any(feature = "server", test))]
pub mod app_config;

#[cfg(any(feature = "server", test))]
pub mod book_existance_checker;

#[cfg(any(feature = "server", test))]
mod handlers;

#[cfg(any(feature = "server", test))]
pub mod reservations_repository;
