# Solution Steps
1. Build the Project & Generate Protobuf Code

- Run cargo build to compile the project and automatically convert messages.proto into messages.rs.
- This ensures all Protobuf message definitions (e.g., EchoMessage, AddRequest) are up to date.
2. Activate Logger 
- Add ``` env_logger = "0.11.6"``` crate in cargo.toml file for log backend
- Add ```let _ = env_logger::try_init();``` in each test case to be sure to Initialize the logger
- Set RUST_LOG environment variable for detailed logs:
	- Write ```$env:RUST_LOG="trace";``` in cmd so you can see all trace-level logs in your console.

3. Test Execution
- Use : ```cargo test -- --test-threads=1```.
- This runs tests sequentially (one thread), reducing log interference among tests.
- Parallel logs from multiple tests can be interleaved, making debugging harder.

4. Bug Analysis and Fix 
- Inspected test logs and debugged the server code to identify key issues.
- Detailed findings are in [Bug Analysis and Fix Report](Reports/Bug_Analysis_and_Fix_Report.md).
- This document outlines each discovered bug, its root cause, and the applied fix.

5. Enhancing the Single-Threaded Server
- Modified the existing server code to handle multiple clients by:
	- Looping over all incoming connections (in non-blocking mode).
	- Storing them in a ```Vec<Client>``` and iterating to serve each client in round-robin style.
- This approach ensures basic concurrency in a single-thread environment (time-sliced handling of multiple clients).

6. Implementing the Multi-Threaded Server
- Created [multithreaded_server.rs](src/multithreaded_server.rs) to spawn dedicated threads for each client.
- Each client’s connection runs in its own thread, improving responsiveness and concurrency compared to single-threaded.
7. New Test File for Multithreading
- Introduced [client_test_multithreading](tests/client_test_multithreading.rs) to verify the threaded server.
- Validates concurrency scenarios (e.g., multiple clients sending messages in parallel).

8. Performance & Observations
- The multithreaded server passes all tests.
- It is faster than single-threaded when handling multiple clients concurrently as noticed from test time. 

9. Documentation Generation

- Documented the multi-threaded server and server files using ```cargo doc --document-private-items```.
- This command generates documentation for private items as well.
- Use ```cargo doc --open --document-private-items``` to open the documentation in the browser or navigate to open the [index.html](target/doc/embedded_recruitment_task/multithreaded_server/index.html) file from target/doc/embedded_recruitment_task/multithreaded_server path.

10. Additional Test Cases
 - Added test cases for the server in the [Additional_Test_Cases_Report file](Reports/Additional_Test_Cases_Report.md ) .
 - The added tests is in the below of the test cases files in the [client_test_multithreading](tests/client_test_multithreading.rs) and [client_test](tests/client_test.rs) files.

11. Tests Evidence (Logs) 
- Attached logs for both servers:
	- [Single_Threaded_Server_Test_Evidence](Reports/Single_Threaded_Server_Test_Evidence.txt)
	- [Multi_Threaded_Server_Test_Evidence](Reports/Multi_Threaded_Server_Test_Evidence.txt) .

12.  Architectural Flaws & How They Were Addressed [Report](Reports/Architectural_Flaws.md)



## Conclusion:

With these steps, we have:

- A single-threaded server capable of round-robin handling multiple clients.
- A multithreaded server that runs each client in seperate thread.
- Test evidence (logs) and bug fix documentation.
- Updated documentation for code via cargo doc.


## **Repository Structure**
```plaintext
.
├── proto/
│   └── messages.proto                     # Protocol buffer file defining messages
├── Reports/
│   ├── Bug_Analysis_and_Fix_Report.md     # Identified bugs and fixes
│   ├── Architectural_Flaws.md              # Analysis of architectural issues and solutions
│   ├── Multi_Threaded_Server_Test_Evidence.txt  # Logs for multi-threaded server
│   ├── Single_Threaded_Server_Test_Evidence.txt # Logs for single-threaded server
│   └── Additional_Test_Cases_Report.md     # Test cases added for validation
├── src/
│   ├── lib.rs                             # Core server logic
│   ├── multithreaded_server.rs            # Multi-threaded server implementation
│   └── server.rs                          # Single-threaded server implementation
├── tests/
│   ├── client_test.rs                     # Tests for single-threaded server
│   ├── client_test_multithreading.rs      # Tests for multi-threaded server
│   ├── client.rs                          # Client implementation
├── build.rs                               # Build script for Protobuf handling
├── Cargo.toml                             # Rust package configuration
├── README.md                              # Repository documentation
├── SOLUTION.md                            # Detailed solution steps
└── target/                                # Compiled outputs and documentation
```
