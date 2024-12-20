Below is a concise, step-by-step project plan:

**Project Overview**  
Create a headless Rust-based daemon that:  
- Discovers all Sonos devices on the local network.  
- Monitors currently playing tracks and detects changes.  
- Scrobbles tracks to Last.fm by updating "now playing" and then submitting a "scrobble" after a playback threshold.  
- Ensures proper credential storage, logging, and reliability (backoff/retry on errors, local caching).

**Key Components**  
1. **Discovery & Device Interaction**  
   - Use Sonos APIs to discover all zones and subscribe to playback events.  
   - Poll or subscribe to state changes to detect track start, mid-listening, and track end.

2. **Last.fm Integration**  
   - Implement OAuth/token-based auth flow and store credentials in `.env` or `~/.config`.  
   - Use `track.updateNowPlaying` on track start, and `track.scrobble` once the track meets a listening threshold.  
   - Include error handling, retries with backoff, and rate-limiting.

3. **Data Handling**  
   - Maintain a SQLite database to log previously scrobbled tracks and avoid duplicates.  
   - Store minimal session data (last scrobbled track, timestamps).

4. **Resilience & Logging**  
   - Implement retry logic for API failures.  
   - Use a configurable log level and standard plain text logs for debugging.  
   - Gracefully handle network outages and Sonos/API timeouts.

5. **Testing & Validation**  
   - Aim for comprehensive unit test coverage, including mocks for Sonos and Last.fm services.  
   - Implement integration tests against real devices and test Last.fm endpoints with test accounts.

6. **Configuration & Deployment**  
   - Single binary crate with well-structured modules (no separate library crate needed).  
   - Credentials, endpoints, and options set via `.env` or `~/.config`.  
   - Support running as a systemd service, Docker container, or just a standalone binary.

7. **Performance & Cleanup**  
   - Review CPU and memory usage for running on low-power devices (e.g., Raspberry Pi).  
   - Provide CLI arguments for logging level, polling intervals, and dry-run mode.

**Implementation Steps**  
1. **Setup & Boilerplate**:  
   - Initialize a Rust project with a `main.rs` and basic configuration parsing.  
   - Set up logging framework and environment variable reading.

2. **Sonos Integration**:  
   - Implement Sonos discovery (UPnP, SSDP) and event subscription logic.  
   - Parse currently playing track info from device responses.

3. **Last.fm Authentication**:  
   - Implement credential loading from `.env`/config file.  
   - Write a small routine to authenticate and refresh tokens if needed.

4. **Scrobble Logic**:  
   - On track start: call `track.updateNowPlaying`.  
   - At half track length or 4 mins: call `track.scrobble`.  
   - Store scrobble event in SQLite to avoid duplicates.

5. **Persistence & Logging**:  
   - Integrate SQLite for track history.  
   - Add detailed logging for each step.

6. **Testing**:  
   - Unit test Sonos discovery logic with mocks.  
   - Unit test Last.fm API calls with mock responses.  
   - Integration test against a real Sonos setup and test Last.fm accounts.

7. **Deployment & Documentation**:  
   - Write systemd service file for headless operation.  
   - Optional Dockerfile.  
   - Document configuration, runtime flags, and troubleshooting steps.

8. **Final Review & Optimization**:  
   - Check code formatting, linting, and performance.  
   - Ensure 100% test coverage.  
   - Validate reliability with real usage scenarios.

**Outcome**  
A robust, headless Rust daemon that continuously listens to Sonos activity and reliably scrobbles played tracks to Last.fm, with solid testing, logging, and deployment options ready for any environment.
