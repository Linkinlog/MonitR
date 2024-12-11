#### **Phase 1: Basic Uptime Logger**

- [x]  Initialize a Rust project.
- [x]  Use `sysinfo` or `uptime_lib` to get system uptime.
- [x]  Log uptime to a local file at regular intervals.
- [x]  Create a CLI interface for viewing the current uptime.

#### **Phase 2: Network Availability Monitoring**

- [x]  Use `tokio` to implement asynchronous network checks.
- [x]  Ping external endpoints and log response times.
- [ ]  Track success and failure rates over time.

#### **Phase 3: Resource Usage Monitoring**

- [x]  Use `sysinfo` to monitor CPU, memory, and disk usage.
- [x]  Log resource usage trends periodically.
- [ ]  Aggregate data for trend analysis (e.g., calculate min, max, average usage).

#### **Phase 4: HTTP API**

- [ ]  Use `warp` or `axum` to create a simple HTTP server.
- [ ]  Add endpoints for:
    - Uptime: `/system`
    - Network stats: `/network`
    - Resource usage: `/resources`
- [ ]  Serve JSON data for export.

#### **Phase 5: Polish and Extras**

- [x]  Add a feature to log data to a database (e.g., SQLite or PostgreSQL).
- [ ]  Implement graph generation (e.g., CPU usage trends over time) via a web UI or API.
- [ ]  Add authentication to protect sensitive endpoints.
