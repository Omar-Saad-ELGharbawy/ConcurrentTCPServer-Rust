//! A multithreaded TCP server implementation that handles multiple client connections concurrently.
//! 
//! This server implementation uses Rust's threading capabilities to handle each client
//! in a separate thread, allowing for true concurrent processing of client requests.
//! The server supports two types of messages:
//! * Echo messages - Returns the received message back to the client
//! * Add requests - Adds two numbers and returns the result
//!
//! # Architecture
//! * Main server thread accepts new connections
//! * Each client connection is handled in a separate thread
//! * Thread synchronization is handled using Arc and Mutex
//! * Non-blocking I/O is used for connection acceptance

use crate::message::{
    client_message,
    server_message,
    AddResponse,
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
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

/// Represents a connected client and handles its message processing
struct Client {
    /// The TCP stream connected to the client
    stream: TcpStream,
}

impl Client {
    /// Creates a new Client instance
    ///
    /// # Arguments
    /// * `stream` - The TCP stream connected to the client
    ///
    /// # Returns
    /// A new Client instance wrapping the provided stream
    pub fn new(stream: TcpStream) -> Self {
        Client { stream }
    }

    /// Handles client communication by reading and processing messages
    ///
    /// # Returns
    /// * `Ok(())` if message processing was successful or no data was available
    /// * `Err(...)` if the client disconnected or an error occurred
    pub fn handle(&mut self) -> io::Result<()> {
        // Buffer for reading client messages
        let mut buffer = [0; 512];

        // Attempt to read data from the client
        match self.stream.read(&mut buffer) {
            // Handle client disconnection (0 bytes read)
            Ok(0) => {
                info!("Client disconnected.");
                return Err(io::Error::new(
                    io::ErrorKind::ConnectionAborted,
                    "Client disconnected",
                ));
            }

            // Handle successful read of data
            Ok(bytes_read) => {
                // Attempt to decode the received bytes as a ClientMessage
                match ClientMessage::decode(&buffer[..bytes_read]) {
                    Ok(client_msg) => {
                        // Process different message types
                        match client_msg.message {
                            // Handle Echo message
                            Some(client_message::Message::EchoMessage(echo)) => {
                                info!("Received EchoMessage: {}", echo.content);
                                // Create response message
                                let response = ServerMessage {
                                    message: Some(server_message::Message::EchoMessage(echo)),
                                };
                                // Encode and send response
                                let payload = response.encode_to_vec();
                                self.stream.write_all(&payload)?;
                                self.stream.flush()?;
                            }

                            // Handle Add request
                            Some(client_message::Message::AddRequest(add_req)) => {
                                info!("Received AddRequest: a={}, b={}", add_req.a, add_req.b);
                                // Calculate sum
                                let sum = add_req.a + add_req.b;
                                // Create response message
                                let response = ServerMessage {
                                    message: Some(server_message::Message::AddResponse(
                                        AddResponse { result: sum },
                                    )),
                                };
                                // Encode and send response
                                let payload = response.encode_to_vec();
                                self.stream.write_all(&payload)?;
                                self.stream.flush()?;
                            }

                            // Handle empty message
                            None => {
                                error!("Received ClientMessage with no inner message.");
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to decode ClientMessage: {}", e);
                    }
                }
                Ok(())
            }

            // Handle non-blocking case (no data available)
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                Ok(())
            }

            // Handle other errors
            Err(e) => {
                error!("Failed to read from client: {}", e);
                Err(e)
            }
        }
    }
}

/// Type alias for the list of client thread handles
type ClientList = Arc<Mutex<Vec<thread::JoinHandle<()>>>>;

/// Main server struct that handles incoming connections
pub struct Server {
    /// TCP listener for accepting new connections
    listener: TcpListener,
    /// Flag indicating if the server is running
    is_running: Arc<AtomicBool>,
    /// List of handles to client threads
    client_threads: ClientList,
}

impl Server {
    /// Creates a new server instance bound to the specified address
    ///
    /// # Arguments
    /// * `addr` - The address to bind to (e.g., "localhost:8080")
    ///
    /// # Returns
    /// * `Ok(Server)` if server creation was successful
    /// * `Err(...)` if binding to the address failed
    pub fn new(addr: &str) -> io::Result<Self> {
        // Create and bind TCP listener
        let listener = TcpListener::bind(addr)?;
        // Create atomic boolean for server state
        let is_running = Arc::new(AtomicBool::new(false));
        // Create thread-safe vector for client threads
        let client_threads = Arc::new(Mutex::new(Vec::new()));
        
        Ok(Server {
            listener,
            is_running,
            client_threads,
        })
    }

    /// Runs the server, accepting and handling client connections
    ///
    /// # Returns
    /// * `Ok(())` if server shutdown was clean
    /// * `Err(...)` if a fatal error occurred
    pub fn run(&self) -> io::Result<()> {
        // Set server as running
        self.is_running.store(true, Ordering::SeqCst);
        info!("Server is running on {}", self.listener.local_addr()?);
        println!("Server is running on {}", self.listener.local_addr()?);

        // Set listener to non-blocking mode
        self.listener.set_nonblocking(true)?;

        // Main server loop
        while self.is_running.load(Ordering::SeqCst) {
            // Accept new connections
            match self.listener.accept() {
                Ok((stream, addr)) => {
                    info!("New client connected: {}", addr);
                    
                    // Clone Arc for the new thread
                    let is_running = Arc::clone(&self.is_running);
                    
                    // Spawn new thread for client
                    let handle = thread::spawn(move || {
                        let mut client = Client::new(stream);
                        // Client handling loop
                        while is_running.load(Ordering::SeqCst) {
                            if let Err(e) = client.handle() {
                                match e.kind() {
                                    ErrorKind::ConnectionAborted => {
                                        info!("Client {} disconnected.", addr);
                                        break;
                                    }
                                    _ => {
                                        error!("Error handling client {}: {}", addr, e);
                                        break;
                                    }
                                }
                            }
                            thread::sleep(Duration::from_millis(10));
                        }
                    });

                    // Store thread handle
                    self.client_threads.lock().unwrap().push(handle);
                }

                // Handle non-blocking case
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(100));
                }

                // Handle other errors
                Err(e) => {
                    error!("Error accepting connection: {}", e);
                }
            }

            // Clean up completed client threads
            let mut threads = self.client_threads.lock().unwrap();
            threads.retain(|handle| !handle.is_finished());
        }

        // Clean shutdown: wait for all client threads
        let mut threads = self.client_threads.lock().unwrap();
        for handle in threads.drain(..) {
            if let Err(e) = handle.join() {
                error!("Error joining client thread: {:?}", e);
            }
        }

        info!("Server stopped.");
        Ok(())
    }

    /// Stops the server and performs cleanup
    ///
    /// This method:
    /// 1. Sets the running flag to false
    /// 2. Waits for all client threads to complete
    /// 3. Cleans up resources
    pub fn stop(&self) {
        if self.is_running.load(Ordering::SeqCst) {
            // Signal server to stop
            self.is_running.store(false, Ordering::SeqCst);
            info!("Shutdown signal sent.");

            // Wait for client threads to complete
            let mut threads = self.client_threads.lock().unwrap();
            for handle in threads.drain(..) {
                if let Err(e) = handle.join() {
                    error!("Error joining client thread: {:?}", e);
                }
            }
        } else {
            warn!("Server was already stopped or not running.");
        }
    }
}
