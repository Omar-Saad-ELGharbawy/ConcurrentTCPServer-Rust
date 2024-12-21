use crate::message::EchoMessage;
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

        // Use a match to handle non-blocking I/O on Windows
        match self.stream.read(&mut buffer) {
            Ok(0) => {
                /*
                If the client has disconnected (read returned 0 bytes), we now return an error so the server loop breaks.
                Previously, we returned Ok(()), which caused an infinite loop because the server kept calling client.handle() indefinitely.
                */
                info!("Buffer is Empty.");
                
                // Return an Err(...) so the server's inner loop sees an error and breaks out instead of continuing forever.
                return Err(io::Error::new(
                    io::ErrorKind::ConnectionAborted,
                    "Client disconnected",
                ));
                // Old Code
                // return Ok(());
            }
            // We successfully read 'n' bytes, attempt to decode
            Ok(bytes_read) => {
                if let Ok(message) = EchoMessage::decode(&buffer[..bytes_read]) {
                    info!("Received: {}", message.content);
                    // Echo back the message
                    let payload = message.encode_to_vec();
                    self.stream.write_all(&payload)?;
                    self.stream.flush()?;
                } else {
                    error!("Failed to decode message");
                }

                Ok(())
            }
            // If the socket would block, it means no data is ready yet.
            // This is NOT a fatal error on a non-blocking socket.
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                // No data available yet
                info!("No data available yet.");
                // Return Ok(()) so the server can try again later
                return Ok(());
            }
            Err(e) => {
                error!("Failed to read from client: {}", e);
                return Err(e);
            }
        }

        // Read data from the client
        /*
        If the client has disconnected (read returned 0 bytes), we now return an error so the server loop breaks.
        Previously, we returned Ok(()), which caused an infinite loop because the server kept calling client.handle() indefinitely.
        */

        // let bytes_read = self.stream.read(&mut buffer)?;
        // if bytes_read == 0 {
        //     info!("Client disconnected.");
        //     // Return an Err(...) so the server's inner loop sees an error and breaks out instead of continuing forever.
        //     return Err(io::Error::new(
        //         io::ErrorKind::ConnectionAborted,
        //         "Client disconnected",
        //     ));
        //     // Old Code
        //     // return Ok(());
        // }

        // if let Ok(message) = EchoMessage::decode(&buffer[..bytes_read]) {
        //     info!("Received: {}", message.content);
        //     // Echo back the message
        //     let payload = message.encode_to_vec();
        //     self.stream.write_all(&payload)?;
        //     self.stream.flush()?;
        // } else {
        //     error!("Failed to decode message");
        // }

        // Ok(())
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

        // Set the listener to non-blocking mode
        self.listener.set_nonblocking(true)?;

        while self.is_running.load(Ordering::SeqCst) {
            info!("Listening for new clients");
            match self.listener.accept() {
                Ok((stream, addr)) => {
                    info!("New client connected: {}", addr);

                    // Handle the client request
                    let mut client = Client::new(stream);
                    while self.is_running.load(Ordering::SeqCst) {
                        //make match if e is Client disconnected make info Client disconnected and break else make error e
                        if let Err(e) = client.handle() {
                            match e.kind() {
                                ErrorKind::ConnectionAborted => {
                                    info!("Client disconnected.");
                                    break;
                                }
                                _ => {
                                    error!("Error handling client: {}", e);
                                    break;
                                }
                            }
                        }
                        // if let Err(e) = client.handle() {
                        //     error!("Error handling client: {}", e);
                        //     break;
                        // }
                    }
                }
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    // No incoming connections, sleep briefly to reduce CPU usage
                    thread::sleep(Duration::from_millis(100));
                }
                Err(e) => {
                    error!("Error accepting connection: {}", e);
                }
            }
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
