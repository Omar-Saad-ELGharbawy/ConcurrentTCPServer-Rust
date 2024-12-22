// use crate::message::EchoMessage;
use crate::message::{
    client_message,
    server_message,
    AddResponse,
    // These are your generated Protobuf types
    ClientMessage,
    ServerMessage,
};
use log::{error, info, warn};
use prost::Message;
use std::{
    io::{self, ErrorKind, Read, Write},
    net::{TcpListener, TcpStream},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

struct Client {
    stream: TcpStream,
}

impl Client {
    pub fn new(stream: TcpStream) -> Self {
        Client { stream }
    }

    pub fn handle(&mut self) -> io::Result<()> {
        let mut buffer = [0; 512];

        // Read data from the client

        // Use a match to handle non-blocking I/O on Windows (FIXED)
        match self.stream.read(&mut buffer) {
            // (A) The client disconnected (0 bytes read)
            Ok(0) => {
                /*
                If the client has disconnected (read returned 0 bytes), we now return an error so the server loop breaks.
                Previously, we returned Ok(()), which caused an infinite loop because the server kept calling client.handle() indefinitely.
                */
                info!("Buffer is Empty.");

                // Return an Err(...) so the server's inner loop sees an error and breaks out instead of continuing forever. (FIXED)
                return Err(io::Error::new(
                    io::ErrorKind::ConnectionAborted,
                    "Client disconnected",
                ));
                // Old Code
                // return Ok(());
            }
            // (B) We successfully read bytes, attempt to decode (successful read)
            Ok(bytes_read) => {
                // 1. Decode the data as a `ClientMessage` instead of just `EchoMessage`. (FIXED)
                match ClientMessage::decode(&buffer[..bytes_read]) {
                    Ok(client_msg) => {
                        // 2. Match on the `oneof` to see if it’s EchoMessage or AddRequest
                        match client_msg.message {
                            Some(client_message::Message::EchoMessage(echo)) => {
                                info!("Received EchoMessage: {}", echo.content);
                                // Build a ServerMessage with EchoMessage
                                let response = ServerMessage {
                                    message: Some(server_message::Message::EchoMessage(echo)),
                                };
                                // Encode and write the response back
                                let payload = response.encode_to_vec();
                                self.stream.write_all(&payload)?;
                                self.stream.flush()?;
                            }
                            Some(client_message::Message::AddRequest(add_req)) => {
                                info!("Received AddRequest: a={}, b={}", add_req.a, add_req.b);
                                // Calculate the sum
                                let sum = add_req.a + add_req.b;
                                // Build a ServerMessage with AddResponse
                                let response = ServerMessage {
                                    message: Some(server_message::Message::AddResponse(
                                        AddResponse { result: sum },
                                    )),
                                };
                                // Encode and write the response back
                                let payload = response.encode_to_vec();
                                self.stream.write_all(&payload)?;
                                self.stream.flush()?;
                            }
                            None => {
                                // The oneof was empty - rare, but we can log and ignore
                                error!("Received ClientMessage with no inner message.");
                            }
                        }
                    }
                    Err(e) => {
                        // If decoding fails, log an error
                        error!("Failed to decode ClientMessage: {}", e);
                    }
                }
                // Return Ok after handling the message
                Ok(())
            }
            // (C) If the socket would block, it means no data is ready yet.
            // This is NOT a fatal error on a non-blocking socket. (FIXED)
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                // No data available yet
                // info!("No data available yet.");
                // Return Ok(()) so the server can try again later
                return Ok(());
            }
            // (D) Other I/O errors
            Err(e) => {
                error!("Failed to read from client: {}", e);
                return Err(e);
            }
        }
    }
}

pub struct Server {
    listener: TcpListener,
    is_running: Arc<AtomicBool>,
}

impl Server {
    /// Creates a new server instance
    pub fn new(addr: &str) -> io::Result<Self> {
        let listener = TcpListener::bind(addr)?;
        let is_running = Arc::new(AtomicBool::new(false));
        Ok(Server {
            listener,
            is_running,
        })
    }

    /// Runs the server, listening for incoming connections and handling them
    pub fn run(&self) -> io::Result<()> {
        self.is_running.store(true, Ordering::SeqCst); // Set the server as running
        info!("Server is running on {}", self.listener.local_addr()?);
        println!("Server is running on {}", self.listener.local_addr()?);

        // Set the listener to non-blocking mode
        self.listener.set_nonblocking(true)?;

        // Vector of Clients to hold active clients in it to serve them later
        let mut clients: Vec<Client> = Vec::new();

        while self.is_running.load(Ordering::SeqCst) {
            // 1) Accept new clients if available
            info!("Listening for new clients");
            // println!("Listening for new clients");
            match self.listener.accept() {
                Ok((stream, addr)) => {
                    info!("New client connected: {}", addr);
                    // Push the new client into our collection (FIXED)
                    clients.push(Client::new(stream));
                }
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    // No incoming connections, sleep briefly to reduce CPU usage
                    thread::sleep(Duration::from_millis(100));
                }
                Err(e) => {
                    error!("Error accepting connection: {}", e);
                }
            }
            // 2) Round-robin: handle each connected client once per iteration in single threaded approach (FIXED)
            let mut i = 0;
            while i < clients.len() {
                if let Err(e) = clients[i].handle() {
                    // match if error is Client disconnected make info Client disconnected and break else make error e
                    match e.kind() {
                        ErrorKind::ConnectionAborted => {
                            info!("Client disconnected.");
                            // Remove the client from the list
                            clients.remove(i);
                            break;
                        }
                        _ => {
                            error!("Error handling client: {}", e);
                            // Remove the client from the list
                            clients.remove(i);
                            break;
                        }
                    }
                } else {
                    i += 1;
                }
            }
            // 3) Sleep to avoid busy-spinning
            thread::sleep(Duration::from_millis(50));
        }

        info!("Server stopped.");
        Ok(())
    }

    /// Stops the server by setting the `is_running` flag to `false`
    pub fn stop(&self) {
        if self.is_running.load(Ordering::SeqCst) {
            self.is_running.store(false, Ordering::SeqCst);
            info!("Shutdown signal sent.");
        } else {
            warn!("Server was already stopped or not running.");
        }
    }
}
