> [!WARNING]  
> WIP

# Sonos Scrobbler 🎶

## Summary of Project 📚
The **Sonos Scrobbler** is a headless Rust-based daemon designed to interact with Sonos devices on your local network. It discovers all Sonos devices, monitors the currently playing tracks, and scrobbles them to [Last.fm](https://www.last.fm/) to keep track of your listening habits. Aimed at enhancing the Sonos experience, this project ensures proper credential storage, logging, and resilience in internet connectivity.

### Features:
- Discover Sonos devices on the network
- Monitor track changes and log them
- Submit listening data to Last.fm
- SQLite database to track scrobbled songs and prevent duplicates
- Fully configurable via `.env` files

## How to Use 🚀
1. **Clone the Repository**
   ```bash
   git clone https://github.com/harperreed/sonos_scrobbler.git
   cd sonos_scrobbler
   ```

2. **Set Up Your Environment**
   - Create a `.env` file using the example provided:
     ```bash
     cp .env.example .env
     ```
   - Insert your Last.fm API credentials in the `.env` file.

3. **Build the Project**
   Ensure you have Rust installed. Then, run the following command to build the project:
   ```bash
   cargo build --release
   ```

4. **Run the Application**
   Execute the binary to start listening for your Sonos devices and begin scrobbling!
   ```bash
   cargo run --release
   ```

5. **Stop the Daemon**
   To gracefully stop the daemon, use `Ctrl+C`.

## Tech Info 💻
- **Programming Language**: Rust (Edition 2021)
- **Key Dependencies**:
  - `sqlx`: To interact with the SQLite database
  - `rusty-sonos`: For Sonos device discovery and communication
  - `tokio`: Asynchronous runtime for building concurrent applications
  - `log` and `env_logger`: For structured and configurable logging
  - `anyhow`: Error handling for easier management of results and errors

- **Directory Structure**:
```
src/
  ├── sonos/
  │   ├── database.rs         # Database interactions
  │   ├── discovery.rs        # Device discovery logic
  │   ├── events.rs           # Event polling and handling
  │   └── mod.rs              # Module re-exporting
  ├── lib.rs                  # Library root
  └── main.rs                 # Application entry point
.env.example                   # Example environment configuration
Cargo.toml                     # Cargo manifest
.gitignore                     # Files to ignore
```

### Running Tests 🧪
To ensure everything works correctly, you can run the tests included with the project:
```bash
cargo test
```

---

For any issues, improvements, or feature requests, please check [issues.md](issues.md), or contribute directly to the project. Happy scrobbling! 🎉
