# Solution Steps
1. cargo build to build project and convert messages.proto file into messages.rs
2. Activate Logger 
	1. Add  env_logger = "0.11.6" crate in cargo.toml file for log backend
	2. Add let _ = env_logger::try_init(); in each test case to be sure to init logger
	3. Write $env:RUST_LOG="trace"; in cmd to turn on all logging for the application
3. Running tests using cargo test -- --test-threads=1 to run the tests using one thread not in parallel to prevent the tests and log messages from interfere with each other.
4. Started analyzing the test logs and debugging the server code to find bugs and fix it as in [Bug Analysis and Fix Report](../Bug_Analysis_and_Fix_Report.md) .  

5. Fixed the single threaded server to handle multiple clients by looping over the incoming connections and storing them in vector then handling them in a loop.

6. Started implementing the multi-threaded server in server_multithreadi.

Here you can document all bugs and design flaws.