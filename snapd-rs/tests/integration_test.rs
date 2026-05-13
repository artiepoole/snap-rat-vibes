//! Integration tests for snapd-rs API endpoints
//!
//! Tests use a mock snapd server that runs in a separate thread.
//! Each test gets its own isolated mock server instance.

mod mock_snapd;

use snapd_rs::SnapdClient;

/// Test listing snaps - with mock snapd
#[tokio::test]
async fn test_list_snaps() {
    let mock = mock_snapd::MockSnapd::start().expect("Failed to start mock snapd");
    let client = SnapdClient::with_socket(mock.socket_path().to_string_lossy().to_string());

    let result = client.list_snaps().await;
    assert!(result.is_ok(), "list_snaps failed: {:?}", result.err());

    let snaps = result.unwrap();
    assert!(!snaps.is_empty(), "Expected at least one snap");
    println!("✓ list_snaps: found {} snaps", snaps.len());
}

/// Test system info - with mock snapd
#[tokio::test]
async fn test_system_info() {
    let mock = mock_snapd::MockSnapd::start().expect("Failed to start mock snapd");
    let client = SnapdClient::with_socket(mock.socket_path().to_string_lossy().to_string());

    let result = client.get_system_info().await;
    assert!(result.is_ok(), "get_system_info failed: {:?}", result.err());

    let info = result.unwrap();
    println!("✓ system_info: version={}", info.version);
}

/// Test app listing - with mock snapd
#[tokio::test]
async fn test_list_apps() {
    let mock = mock_snapd::MockSnapd::start().expect("Failed to start mock snapd");
    let client = SnapdClient::with_socket(mock.socket_path().to_string_lossy().to_string());

    let result = client.list_apps().await;
    assert!(result.is_ok(), "list_apps failed: {:?}", result.err());

    println!("✓ list_apps: endpoint accessible");
}
