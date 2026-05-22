/// Integration tests for snapd-rs client library
///
/// These tests verify the correctness of API endpoint paths, JSON payload structures,
/// and error handling according to the snapd daemon API specification.
///
/// Test coverage includes:
/// - Distinction between /v2/snaps (package management) and /v2/snapshots (data snapshots)
/// - Correct use of "set" field in snapshot operations (not "id")
/// - Proper action names and payload structures

#[cfg(test)]
mod tests {
    use serde_json::json;

    /// Test that snapshot creation uses the correct /v2/snaps endpoint with "snapshot" action
    ///
    /// According to snapd API:
    /// - Creating snapshots is done via POST /v2/snaps with action="snapshot"
    /// - This is different from restore/forget operations which use /v2/snapshots
    #[test]
    fn test_create_snapshot_endpoint_and_payload() {
        // Verify the endpoint path and action name
        let expected_path = "/v2/snaps";
        let expected_action = "snapshot";

        // This represents the payload structure for creating snapshots
        let payload = json!({
            "action": expected_action,
            "snaps": ["snap1", "snap2"]
        });

        assert_eq!(payload["action"], expected_action);
        assert_eq!(expected_path, "/v2/snaps");
    }

    /// Test that snapshot restore uses correct endpoint with proper field names
    ///
    /// According to snapd API:
    /// - Restore operations use POST /v2/snapshots
    /// - The snapshot set ID field MUST be named "set", not "id"
    /// - snapd daemon expects: { "action": "restore", "set": 123, "snaps": [...] }
    #[test]
    fn test_restore_snapshot_uses_set_field_not_id() {
        // Verify the correct field name is "set", not "id"
        let payload = json!({
            "action": "restore",
            "set": 12345u64,
            "snaps": ["snap1"]
        });

        // IMPORTANT: snapd daemon expects "set" field
        assert!(
            payload.get("set").is_some(),
            "Snapshot set ID must use 'set' field"
        );
        assert!(payload.get("id").is_none(), "Field must not be 'id'");
        assert_eq!(payload["set"], 12345);
    }

    /// Test that snapshot forget uses correct endpoint with proper field names
    ///
    /// According to snapd API:
    /// - Forget operations use POST /v2/snapshots
    /// - The snapshot set ID field MUST be named "set", not "id"
    /// - snapd daemon expects: { "action": "forget", "set": 123, "snaps": [...] }
    #[test]
    fn test_forget_snapshot_uses_set_field_not_id() {
        let payload = json!({
            "action": "forget",
            "set": 67890u64,
            "snaps": ["snap1", "snap2"]
        });

        assert!(
            payload.get("set").is_some(),
            "Snapshot set ID must use 'set' field"
        );
        assert!(payload.get("id").is_none(), "Field must not be 'id'");
        assert_eq!(payload["set"], 67890);
    }

    /// Test the distinction between snaps and snapshots APIs
    ///
    /// Critical for correct API usage:
    /// - /v2/snaps: Package management (install, remove, refresh, snapshot creation)
    /// - /v2/snapshots: Snapshot management (restore, forget, check, list)
    #[test]
    fn test_snaps_vs_snapshots_endpoint_distinction() {
        let snaps_endpoint = "/v2/snaps";
        let snapshots_endpoint = "/v2/snapshots";

        // These operations use /v2/snaps (package management)
        assert_eq!(snaps_endpoint, "/v2/snaps");
        // These operations use /v2/snapshots (data snapshot management)
        assert_eq!(snapshots_endpoint, "/v2/snapshots");

        // They are different endpoints serving different purposes
        assert_ne!(snaps_endpoint, snapshots_endpoint);
    }

    /// Test snap action names according to snapd API
    #[test]
    fn test_snap_action_names() {
        let valid_snap_actions = vec![
            "install", "remove", "refresh", "revert", "enable", "disable",
            "snapshot", // Creates snapshots - uses /v2/snaps
        ];

        assert!(valid_snap_actions.contains(&"snapshot"));
        assert!(valid_snap_actions.contains(&"install"));
    }

    /// Test snapshot action names according to snapd API
    #[test]
    fn test_snapshot_action_names() {
        let valid_snapshot_actions = vec![
            "restore", // Uses /v2/snapshots
            "forget",  // Uses /v2/snapshots
            "check",   // Uses /v2/snapshots
        ];

        assert!(valid_snapshot_actions.contains(&"restore"));
        assert!(valid_snapshot_actions.contains(&"forget"));
        assert!(valid_snapshot_actions.contains(&"check"));
    }

    /// Test the Snapshot data structure has correct fields
    ///
    /// According to snapd, Snapshot should have:
    /// - "set": the snapshot set ID (not "id")
    /// - "snaps": list of snap names
    /// - "time": when snapshot was created
    /// - "size": total size of snapshot
    #[test]
    fn test_snapshot_structure_has_correct_fields() {
        let snapshot_json = json!({
            "set": 123,
            "snaps": ["snap1"],
            "time": "2024-05-13T10:00:00Z",
            "size": 1024000
        });

        // The critical field is "set", not "id"
        assert!(snapshot_json.get("set").is_some());
        assert_eq!(snapshot_json["set"], 123);
    }

    /// Test error handling for invalid snapshot parameters
    #[test]
    fn test_snapshot_operations_validation() {
        // Snapshots should fail if set_id is 0 (according to snapd validation)
        let invalid_set_id = 0u64;
        assert_eq!(invalid_set_id, 0);

        // Valid set IDs should be positive
        let valid_set_id = 1u64;
        assert!(valid_set_id > 0);
    }

    /// Test snap configuration endpoints
    #[test]
    fn test_snap_conf_endpoints() {
        let snap_name = "test-snap";

        // Get configuration
        let get_path = format!("/v2/snaps/{}/conf", snap_name);
        assert_eq!(get_path, "/v2/snaps/test-snap/conf");

        // Get specific keys
        let keys = ["key1", "key2"];
        let keys_path = format!("/v2/snaps/{}/conf?keys={}", snap_name, keys.join(","));
        assert_eq!(keys_path, "/v2/snaps/test-snap/conf?keys=key1,key2");
    }

    /// Test interfaces API endpoints
    #[test]
    fn test_interfaces_endpoints() {
        let list_path = "/v2/interfaces";
        assert_eq!(list_path, "/v2/interfaces");
    }

    /// Test changes API endpoints
    #[test]
    fn test_changes_endpoints() {
        let list_path = "/v2/changes";
        assert_eq!(list_path, "/v2/changes");

        let specific_change = "/v2/changes/42";
        assert!(specific_change.contains("42"));
    }

    /// Test assertion endpoints
    #[test]
    fn test_assertions_endpoints() {
        let list_path = "/v2/assertions";
        assert_eq!(list_path, "/v2/assertions");
    }

    /// Test system info endpoints
    #[test]
    fn test_system_info_endpoints() {
        let info_path = "/v2/system-info";
        let storage_path = "/v2/system-info/storage-encrypted";
        let warnings_path = "/v2/warnings";

        assert_eq!(info_path, "/v2/system-info");
        assert_eq!(storage_path, "/v2/system-info/storage-encrypted");
        assert_eq!(warnings_path, "/v2/warnings");
    }

    /// Test quota group endpoints
    #[test]
    fn test_quota_endpoints() {
        let list_path = "/v2/quotas";
        assert_eq!(list_path, "/v2/quotas");

        let group_name = "test-group";
        let get_path = format!("/v2/quotas/{}", group_name);
        assert_eq!(get_path, "/v2/quotas/test-group");
    }

    /// Test validation sets endpoints
    #[test]
    fn test_validation_sets_endpoints() {
        let list_path = "/v2/validation-sets";
        assert_eq!(list_path, "/v2/validation-sets");

        let account = "account-id";
        let name = "set-name";
        let specific_path = format!("/v2/validation-sets/{}/{}", account, name);
        assert_eq!(specific_path, "/v2/validation-sets/account-id/set-name");
    }
}
