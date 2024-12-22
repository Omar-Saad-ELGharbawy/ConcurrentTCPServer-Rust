# Architecture Evolution: Old vs. Enhanced Single-Threaded vs. Multithreaded Server

## 1. Old Single-Threaded Server

### Key Characteristics
- **Accept** new client, then **blocks** in a nested loop serving only **one** client until it disconnects.
- **Non-blocking** `accept()`, but once a client is connected, no other clients can be served concurrently.
- Attempts to **decode only** `EchoMessage`; no support for `AddRequest`.

### Architectural Flaws
1. **Single-Client Blocking**  
   The server’s main loop calls `client.handle()` repeatedly for the same client. New connections are not accepted until the current client disconnects.
2. **Inadequate Decoding**  
   Decodes only `EchoMessage`; if a different message (e.g., `AddRequest`) arrives, decoding fails silently, logging “Failed to decode message.”
3. **Infinite “Client disconnected.” Loop**  
   When `bytes_read == 0`, the server returns `Ok(())`, so the outer loop never breaks—repeatedly logging “Client disconnected.”
4. **No Real Concurrency**  
   Although `accept()` is non-blocking, the rest of the code is effectively single-client at a time.

---

## 2. Enhanced Single-Threaded Server

### Improvements
1. **Round-Robin Handling of Clients**  
   - Maintains a **`Vec<Client>`**:  
     - Accepts **new** connections in non-blocking mode and pushes them into this vector.  
     - Iterates over **all** clients in a round-robin style, calling `client.handle()` once per loop.
   - **Result**: Time-sliced concurrency—no single client monopolizes the server.

2. **Proper Disconnect Handling**  
   - When `bytes_read == 0`, the code now returns an **error** (`ErrorKind::ConnectionAborted`) instead of `Ok(())`.  
   - This causes the server loop to **remove** the client, preventing the infinite disconnection logs.

3. **Decode `ClientMessage`**  
   - The server decodes **both** `EchoMessage` and `AddRequest`, returning the appropriate `ServerMessage`.  
   - Fixes the old issue of ignoring `AddRequest`.

4. **Non-Blocking I/O**  
   - Continues to use non-blocking `accept()`.  
   - If a client’s read call yields `WouldBlock`, the server treats it as “no data yet” and moves on.

### Remaining Limitations
- Still **single-threaded**: while multiple clients are served, heavy or CPU-bound tasks could slow everyone down because only one CPU core is used at a time.

---

## 3. Multithreaded Server

### Additional Enhancements
1. **True Parallel Client Handling**  
   - The server **spawns a thread** for each newly accepted client.  
   - Main thread returns quickly to `accept()` further connections.  
   - Each client is handled **independently**, leveraging multiple CPU cores.

2. **Shared State & Thread Management**  
   - Uses an **`Arc<AtomicBool>`** for `is_running`, so all threads know when to stop.  
   - Maintains a thread-safe structure (e.g., `Arc<Mutex<Vec<JoinHandle<()>>>>`) to store client threads and later join them.

3. **All Fixes from Enhanced Single-Threaded**  
   - **Decodes** `ClientMessage` fully (Echo/Add).  
   - **No infinite** loop on disconnect.  
   - **Non-blocking** listener for quick acceptance.

### Outcome
- **Truly concurrent**: multiple clients can send messages simultaneously without being starved in a round-robin loop.  
- **Better performance** under higher load or CPU-intensive workloads.

---

## Summary of the Evolution

1. **Old Server**  
   - **Flaws**: Single-client blocking, partial decoding, infinite disconnect logs.  
2. **Enhanced Single-Threaded**  
   - **Fixed** decoding to handle multiple message types.  
   - **Round-robin** approach for multiple clients in one thread.  
   - **No** infinite disconnect loops.
3. **Multithreaded**  
   - **Threads** for each client, enabling genuine parallelism.  
   - **Retains** correct handling of messages and disconnect logic.

All these changes ensure the final code can handle **multiple** concurrent clients, properly decode **both** Echo and Add requests, and handle edge cases  gracefully.
