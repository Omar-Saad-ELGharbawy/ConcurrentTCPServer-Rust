This section documents the new test cases added to cover edge cases and concurrency scenarios in our multithreaded server. Each test targets specific aspects of robustness, functionality, or performance.

## 1. `test_concurrent_clients_in_threads`

### Purpose
- Validate that the server can handle **multiple clients** running **in parallel threads** on the client side.
- Each client sends multiple **EchoMessage** requests concurrently, ensuring the server responds correctly without blocking or mixing responses.

### Description
- Spawns **N** client threads (default 4). Each client connects to the server, sends **M** messages (default 3), and validates that the echoed content matches.
- Uses `log::trace!` calls to record each message sent and each response received, making concurrency issues or interleaved logs visible.

### Comparison: Single-Threaded vs. Multithreaded

- **Single-Threaded Server**:  
  - **Outcome**: All concurrent client requests eventually succeeded, verifying that the single-threaded event loop can still cycle through multiple connections.  
  - **Performance**: The total test time was **noticeably longer**, as requests from different clients often queued behind each other in the single loop.

- **Multithreaded Server**:  
  - **Outcome**: Also passed all client requests, returning correct echoes in concurrent mode.  
  - **Performance**: The test completed **significantly faster** because each client is truly handled in parallel by its own thread (or via a thread pool). This demonstrated better scalability and responsiveness.

### Observed Results
- **Result**: In both server architectures, the test passed with correct echoes. However, the **multithreaded server finished in less time**, thanks to handling all clients simultaneously rather than round-robin in a single loop.

## 2. `test_rapid_connect_disconnect`

### Purpose
- Ensure the server can handle **frequent connect/disconnect cycles** from multiple clients in quick succession.
- Stress-test the server’s connection-handling logic to detect issues like resource leaks, timing glitches, or race conditions when clients appear and disappear rapidly.

### Description
1. **Spin up** the server in a separate thread.
2. **Spawn** multiple client threads (`client_count`), each performing several (`iterations`) connect/disconnect cycles:
   - The client connects to the server.
   - Waits a **random short delay** (up to 50 ms).
   - Disconnects immediately.
3. **Join** all client threads to ensure they complete successfully.
4. Finally, **stop** the server and join the server thread.

### Expected Outcome
- The server should gracefully accept new connections and close them without crashing or leaking resources.
- Each client’s rapid connect/disconnect sequence should succeed (no errors on connect or disconnect).
- The server loop remains stable until `server.stop()` is invoked.

### Observed Results
- **Result**: The test **passed**. All clients connected and disconnected smoothly, even under repeated rapid cycles.
- No resource leaks or race conditions were apparent, indicating that the server handles quick connect/disconnect scenarios reliably.


## 3. `test_interleaved_echo_and_add`

### Purpose
- Verify the server correctly handles **mixed** message types (EchoMessage and AddRequest) sent **in sequence** over a **single** client connection.
- Check that each request type yields the correct corresponding response (Echo vs. Add).

### Description
1. **Spin up** the server in a separate thread.
2. Create **one** client and connect it to `"localhost:8080"`.
3. **Send** a sequence of messages:
   - `EchoMessage("Test1")`
   - `AddRequest(2, 3)`
   - `EchoMessage("Test2")`
   - `AddRequest(10, 10)`
4. **Verify** that each request receives the **correct** response type:
   - `EchoMessage(...)` => `EchoMessage(...)`
   - `AddRequest(...)` => `AddResponse(result = a + b)`
5. Disconnect the client and **stop** the server.

### Expected Outcome
- The server correctly decodes the **oneof** fields, returning:
  - `EchoMessage("Test1")` in response to `EchoMessage("Test1")`
  - `AddResponse(5)` in response to `(2 + 3)`
  - `EchoMessage("Test2")` in response to `EchoMessage("Test2")`
  - `AddResponse(20)` in response to `(10 + 10)`
- No mismatched or missing responses.

### Observed Results
- **Result**: The test **passed**. All messages received **matching** response types in the **correct** order. 
- This confirms the server’s ability to handle **interleaved** Echo and Add requests over a **single connection** without confusion or error.
