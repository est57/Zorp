//! Local mock server that answers with saved response examples. It only listens on
//! 127.0.0.1, so other machines cannot reach it.
pub mod routes;
pub mod server;

#[cfg(test)]
mod tests;
