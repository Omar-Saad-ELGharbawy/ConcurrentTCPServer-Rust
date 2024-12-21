// use log::{error, info, warn};

use embedded_recruitment_task::{
    // message::{client_message, server_message, AddRequest, EchoMessage},
    server::Server,
};

fn main() {
    // Start the server
    let server = Server::new("localhost:8080").expect("Failed to start server");
    println!("Server started.");

    server.run().expect("Server encountered an error");

    println!("Stop the server.");
    server.stop();
}
