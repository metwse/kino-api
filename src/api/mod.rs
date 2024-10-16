/// Structs for building and running a server.
mod server;
pub use server::{
    Server,
    ServerBuilder
};

/// Twitter's snowflake algorithm to generate unique ids.
mod snowflake;

/// Module for managing `jwt`'s
mod jwt;

/// Implemets [`Server::serve`].
mod serve;

/// Sets up routes.
mod routes;

/// Auth & limit middlewares.
mod middlewares;

/// Database query structs.
mod database;
