# Bugs and Fixes Table
- [Bug 1](#bug_1)
- [Bug_2](#bug_2)
- [Bug_3](#bug_3)
- [Bug_4](#bug_4)
-

***

## Bug_1
**Infinite "Client Disconnected." Logs** 
### Description 
When a client in `test_client_connection` disconnects from the server, the server logs “Client disconnected.” repeatedly in an infinite loop.
### Test Logs Demonstrating the Bug
```
[2024-12-21T16:23:19Z INFO  embedded_recruitment_task::server] Client disconnected.
[2024-12-21T16:23:19Z INFO  embedded_recruitment_task::server] Client disconnected.
[2024-12-21T16:23:19Z INFO  embedded_recruitment_task::server] Client disconnected.
...
(repeats indefinitely)
```
### Root Cause 
Within the **server** code, the inner loop handling a single client never breaks out when `bytes_read == 0`. Specifically, if `client.handle()` returns `Ok(())` upon reading zero bytes (indicating a disconnected client), the server sees no “error” and continues calling `client.handle()` forever.
- server.rs code before fix: 
```
let bytes_read = self.stream.read(&mut buffer)?;
if bytes_read == 0 {
	info!("Client disconnected.");
	return Ok(());
	}
```
### FIX
When disconnected client (`bytes_read == 0`) let client.handle() return an error so the server’s inner loop breaks. 
- server.rs code after fix: 
```
let bytes_read = self.stream.read(&mut buffer)?;
if bytes_read == 0 {
	info!("Client disconnected.");
	return Err(io::Error::new(
	io::ErrorKind::ConnectionAborted,
	"Client disconnected"
	));
	}
```

## Bug_2 :
**Non-Blocking Socket Returns os error 10035**
### Description
When the server receives a malformed or corrupted Protobuf message, it attempts to parse it using `EchoMessage::decode(...)` (or a similar call) and uses an `unwrap()` on the `Result`. If the decode fails, this causes a panic instead of gracefully handling the error.

### Test Logs Demonstrating the Bug
```
[2024-12-21T16:37:15Z ERROR embedded_recruitment_task::server] Error handling client: A non-blocking socket operation could not be completed immediately. (os error 10035)
```

### Root Cause
On Windows, a non-blocking TcpStream can yield WSAEWOULDBLOCK (error 10035) when no data is immediately available. The current code does not handle ErrorKind::WouldBlock as a normal condition.
### Fix
Check for ErrorKind::WouldBlock in the read call. If encountered, return Ok(()) instead of treating it as an error.And the server will try again late until receiving the data.
- server.rs code after fix:
``` 
match self.stream.read(&mut buffer) {
    Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
		info!("No data available yet.");
        // No data available yet
        // info!("No data available yet.");
        // Return Ok(()) so the server can try again later
        return Ok(());
        }
    }
```

## Bug_3
**Single-Threaded Server Fails to Accept Multiple Clients**
### Description
When multiple clients attempt to connect to the server, the server only accepts the first client.
When a second or third client attempts to connect, the server is still “stuck” handling the first client and waiting its messages.
### Test Logs Demonstrating the Bug
```
test test_multiple_clients ... [2024-12-21T22:37:49Z INFO  client_test::client] Receiving message from the server
[2024-12-21T22:37:49Z INFO  embedded_recruitment_task::server] Server is running on [::1]:8080
[2024-12-21T22:37:49Z INFO  embedded_recruitment_task::server] Listening for new clients
[2024-12-21T22:37:49Z INFO  embedded_recruitment_task::server] New client connected: [::1]:51717
[2024-12-21T22:37:49Z INFO  embedded_recruitment_task::server] Received:
Hello, World!
[2024-12-21T22:37:49Z INFO  client_test::client] Received 17 bytes from the server
[2024-12-21T22:37:49Z INFO  client_test::client] Receiving message from the server
[2024-12-21T22:37:49Z INFO  embedded_recruitment_task::server] No data available yet.
[2024-12-21T22:37:49Z INFO  embedded_recruitment_task::server] No data available yet.
[2024-12-21T22:37:49Z INFO  embedded_recruitment_task::server] No data available yet.
...
(repeats indefinitely)

```
The logs show only one successful "New client connected" message, indicating subsequent clients never get accepted.

### Root Cause
The **server code** blocks on a single client in a loop until that client disconnects. It does not return to `accept()` to check for new connections. As a result, no additional clients can connect while the first client is active.

### Fix
Changed the server’s architecture to handle multiple clients in single threded approach.
Implement a **round-robin** style approach in a single thread:

1. **Set the listener to non-blocking** so `accept()` doesn’t block.  
2. **Store** each connected client in a Vector of CLients to serve them later (e.g., `Vec<Client>`).  
3. **Iterate** over all clients in each loop iteration, calling `client.handle()` once.  
4. **Remove** clients that disconnect or cause an error.  
5. Insert a **sleep** (e.g., `thread::sleep(Duration::from_millis(50))`) at the end of each loop to avoid busy-spinning.
- server.rs code after fix:
``` 

// Vector of Clients to hold active clients in it to serve them later
let mut clients: Vec<Client> = Vec::new();

while self.is_running.load(Ordering::SeqCst) {
	// 1) Accept new clients if available
	info!("Listening for new clients");
	match self.listener.accept() {
		Ok((stream, addr)) => {
			info!("New client connected: {}", addr);
			// Push the new client into our collection
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
	// 2) Round-robin: handle each connected client once per iteration in single threaded approach
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
``` 
## Bug 4
**Server Ignores AddRequest and Only Handles EchoMessage**
### Description
When a client sends an `AddRequest`, the server is unable to decode and respond to it because it only attempts to parse incoming data as an `EchoMessage`. So, the server never sends an `AddResponse`, and any tests expecting an addition result (e.g., `test_client_add_request`) fail or time out.
### Test Logs Demonstrating the Bug
```
test test_client_add_request ... [2024-12-21T23:53:36Z TRACE client_test] 5 : test_client_add_request Start.
[2024-12-21T23:53:36Z INFO  embedded_recruitment_task::server] Server is running on [::1]:8080
[2024-12-21T23:53:36Z TRACE client_test] Client connected to the server.
[2024-12-21T23:53:36Z INFO  embedded_recruitment_task::server] Listening for new clients
[2024-12-21T23:53:36Z TRACE client_test] Client send AddRequest message : AddRequest { a: 10, b: 20 } to the server.
[2024-12-21T23:53:36Z INFO  embedded_recruitment_task::server] New client connected: [::1]:52958
[2024-12-21T23:53:36Z INFO  client_test::client] Receiving message from the server
[2024-12-21T23:53:36Z INFO  embedded_recruitment_task::server] Received: 
[2024-12-21T23:53:37Z INFO  embedded_recruitment_task::server] Listening for new clients
[2024-12-21T23:53:37Z INFO  embedded_recruitment_task::server] Listening for new clients
[2024-12-21T23:53:37Z INFO  embedded_recruitment_task::server] Listening for new clients
...
(repeats indefinitely)
```

### Root Cause
The server code attempts to decode **all** incoming messages strictly as `EchoMessage`, ignoring other variants (like `AddRequest`). 

### Fix
Change the code to decode a **`ClientMessage`** (which is a `oneof` containing either an `EchoMessage` or an `AddRequest`). After decoding, match on the `ClientMessage::Message` variant and:
- If it’s `EchoMessage`, echo it back.
- If it’s `AddRequest`, compute `a + b` and send an `AddResponse`.


- server.rs code before fix:
``` 
if let Ok(message) = EchoMessage::decode(&buffer[..bytes_read]) {
    info!("Received: {}", message.content);
    // Echo it back
    let payload = message.encode_to_vec();
    self.stream.write_all(&payload)?;
    self.stream.flush()?;
} else {
    error!("Failed to decode message");
    // (No other message types handled)
}
``` 
- server.rs code after fix:
``` 
match ClientMessage::decode(&buffer[..bytes_read]) {
    Ok(client_msg) => {
        match client_msg.message {
            Some(client_message::Message::EchoMessage(echo)) => {
                info!("Received EchoMessage: {}", echo.content);
                // Build and send a ServerMessage with EchoMessage
                let response = ServerMessage {
                    message: Some(server_message::Message::EchoMessage(echo)),
                };
                let payload = response.encode_to_vec();
                self.stream.write_all(&payload)?;
                self.stream.flush()?;
            }
            Some(client_message::Message::AddRequest(add_req)) => {
                info!("Received AddRequest: a={}, b={}", add_req.a, add_req.b);
                // Calculate the sum and build the AddResponse
                let sum = add_req.a + add_req.b;
                let response = ServerMessage {
                    message: Some(server_message::Message::AddResponse(AddResponse {
                        result: sum,
                    })),
                };
                let payload = response.encode_to_vec();
                self.stream.write_all(&payload)?;
                self.stream.flush()?;
            }
            None => {
                error!("ClientMessage had no variant (empty oneof).");
            }
        }
    }
    Err(e) => {
        error!("Failed to decode ClientMessage: {}", e);
    }
}
``` 
