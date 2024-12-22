pub mod server;
pub mod multithreaded_server;

pub mod message {
    include!(concat!(env!("OUT_DIR"), "/messages.rs"));
}
