# Integration Tests for snapd-rs

This directory contains integration tests that verify the `snapd-rs` API client using a mock snapd server.
The mock server only changes test infrastructure; it does not change production API response modeling.

## Running Tests

```bash
cd snapd-rs
cargo test --test integration_test
```

With verbose output:
```bash
cargo test --test integration_test -- --nocapture
```

## Test Architecture

### Mock Snapd Server

Each test starts its own mock snapd server that:
- Listens on a unique Unix socket under the OS temp directory (`$TMPDIR` or equivalent)
- Responds to HTTP requests over the socket
- Implements minimal endpoints matching real snapd API
- Automatically stops when the test completes (via Drop implementation)

### Benefits

✓ **Full Isolation** — Each test has its own server instance  
✓ **No External Dependencies** — No need for real snapd installation  
✓ **Fast** — Lightweight HTTP server, no daemon startup overhead  
✓ **Deterministic** — Responses are controlled and predictable  
✓ **Portable** — Works on any platform with Unix sockets  

### Implementation Details

**Mock Server** (`mock_snapd.rs`)
- Single-threaded server using `std::os::unix::net::UnixListener`
- Spawns a background thread per test
- Parses HTTP requests and serves responses
- Automatically cleans up socket files on shutdown

**Test Lifecycle**
1. Test starts mock server: `let mock = MockSnapd::start()?`
2. Test creates client with custom socket: `SnapdClient::with_socket(mock.socket_path())`
3. Test performs assertions
4. Mock server automatically stops when `mock` is dropped
5. Socket file is cleaned up

## Test Coverage

### `test_system_info()` ✓ PASSING
- Tests `GET /v2/system-info` endpoint
- Verifies system metadata parsing (version, architecture, etc.)
- Mock returns: `{version: "2.75.2", series: "24", architecture: "x86_64", ...}`

### `test_list_apps()` ✓ PASSING
- Tests `GET /v2/apps` endpoint
- Verifies app listing and response parsing
- Mock returns: empty array (no apps)

### `test_list_snaps()` ✓ PASSING
- Tests `GET /v2/snaps` endpoint
- Verifies snap listing and response parsing
- Mock returns: one test snap entry

## Adding More Endpoints

To test additional endpoints:

1. Add handler function in `mock_snapd.rs`:
```rust
fn handle_new_endpoint() -> String {
    let body = r#"...json response..."#;
    http_response_ok(body)
}
```

2. Add pattern match in `handle_client()`:
```rust
} else if request.starts_with("GET /v2/new-endpoint") {
    handle_new_endpoint()
```

3. Add test in `integration_test.rs`:
```rust
#[tokio::test]
async fn test_new_endpoint() {
    let mock = MockSnapd::start().expect("Failed to start mock snapd");
    let client = SnapdClient::with_socket(mock.socket_path().to_string_lossy().to_string());
    // ... test assertions
}
```

## Files

- `mock_snapd.rs` — Mock HTTP server implementation (~130 lines)
- `integration_test.rs` — Test cases using mock server (~60 lines)
- `README.md` — This file
