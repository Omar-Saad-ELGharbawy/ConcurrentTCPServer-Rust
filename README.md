# **Concurrent TCP Server - Rust**
## Table of Contents

- [Overview](#overview)
- [Features](#features)
- [Used Technologies](#used-technologies)
- [Solution Steps](#solution-steps)
  - [Bug Fixes & Debugging](#bug-fixes--debugging)
  - [Implementing a Single-Threaded Round-Robin Server](#implementing-a-single-threaded-round-robin-server)
  - [Developing a Multithreaded Server](#developing-a-multithreaded-server)
  - [Concurrency Testing & Validation](#concurrency-testing--validation)
  - [Performance Enhancements & Documentation](#performance-enhancements--documentation)
- [Repository Structure](#repository-structure)
- [Installation & Setup](#installation--setup)
- [Conclusion](#conclusion)
- [Future Enhancements](#future-enhancements)

# **Rust Multithreaded Server**

## **Overview**
This project is a **multithreaded server implemented in Rust** that efficiently handles multiple client connections concurrently. The server was originally a **buggy single-threaded** implementation, and I have successfully transitioned it into a robust, concurrent architecture utilizing Rust’s multithreading capabilities.

This repository includes **the solution steps** detailing the debugging process, concurrency improvements, and architectural enhancements. See [`SOLUTION.md`](SOLUTION.md) for a step-by-step breakdown of the work done.

## **Features**
- **Single-threaded and multithreaded server implementations**
- **Efficient handling of multiple client connections**
- **Thread-safe architecture using Rust’s concurrency primitives**
- **Protobuf-based messaging protocol**
- **Comprehensive test suite for both single-threaded and multithreaded servers**

## **Used Technologies**
- **Rust** for system programming
- **Protobuf** for structured messaging
- **Concurrency primitives** (Threads, Mutex, Channels) for safe multithreading
- **Cargo** for package management
- **Logging & debugging tools** for error tracing

## **Solution Steps**
For a detailed breakdown of the solution, refer to [`SOLUTION.md`](SOLUTION.md). Below is a summary of the key enhancements:

### **Bug Fixes & Debugging**
   - Identified and resolved issues in the initial server implementation.
   - Used logging to trace errors and improve server reliability.

### **Implementing a Single-Threaded Round-Robin Server**
   - Ensured the single-threaded server could handle multiple clients using a non-blocking loop.
   - Implemented a round-robin strategy for handling requests.

### **Developing a Multithreaded Server**
   - Created a separate module (`multithreaded_server.rs`) to implement client handling using **multiple threads**.
   - Assigned **a dedicated thread to each client connection**, improving responsiveness and performance.
   
### **Concurrency Testing & Validation**
   - Added a dedicated test suite (`client_test_multithreading.rs`) to evaluate the performance and stability of the multithreaded server.
   - Verified that multiple clients can send and receive messages concurrently without race conditions.

### **Performance Enhancements & Documentation**
   - Measured performance improvements over the single-threaded implementation.
   - Documented findings and fixes in [`SOLUTION.md`](SOLUTION.md).

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

## **Installation & Setup**
### **Prerequisites**
- Rust (latest stable version recommended) – [Installation Guide](https://www.rust-lang.org/tools/install)
- `protoc` (Protocol Buffers compiler) – [Protocol Buffers Guide](https://protobuf.dev/overview/)

### **Building the Project**
```bash
cargo build
```
This will compile the project and generate the required Protobuf message definitions.

### **Running Tests**
To test the implementation:
```bash
cargo test
```
To run tests sequentially (avoiding log interleaving issues):
```bash
cargo test -- --test-threads=1
```

## **Conclusion**
This project demonstrates the **transition from a single-threaded server to a fully multithreaded concurrent server** in Rust. By addressing architectural flaws and implementing efficient concurrency mechanisms, the final solution achieves **scalability, thread safety, and improved performance**.

For a deeper dive into the solution process, check out [`SOLUTION.md`](SOLUTION.md).

---
### 🚀 **Future Enhancements**
- Implementing **async/await** for better efficiency.
- Adding **load balancing** for client requests.
- Further optimizing **thread pooling** using `tokio` or `rayon`.

Feel free to explore, contribute, or provide feedback!
