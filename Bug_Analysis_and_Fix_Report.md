# Bugs and Fixes Table
- [Bug 1](#bug_1)
- [Bug_2](#bug_2)
-

***

## Bug_1
**Infinite "Client Disconnected." Logs** 
### Description 
When a client in `test_client_connection` disconnects from the server, the server logs “Client disconnected.” repeatedly in an infinite loop. 
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

