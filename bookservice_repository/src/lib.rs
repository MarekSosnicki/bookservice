pub mod api;

#[cfg(any(feature = "client", test))]
pub mod client;

#[cfg(any(feature = "server", test))]
pub mod app_config;
#[cfg(any(feature = "server", test))]
pub mod books_repository;
#[cfg(any(feature = "server", test))]
mod handlers;
