#![allow(dead_code)]
use rucat::metadata::{FileMetadata, PlatformMetadata, UnixMetadata};
#[must_use]
pub fn macos_fixture() -> FileMetadata {
    FileMetadata {
        path: "/tmp/rucat-fixture-test".to_string(),
        size: 12,
        permissions: "-rw-r--r--".to_string(),
        created: Some("Jan  1 10:00:00 2024".to_string()),
        modified: Some("Jan  1 10:00:00 2024".to_string()),
        accessed: Some("Jan  1 10:00:00 2024".to_string()),
        extended_attributes: {
            let mut map = std::collections::HashMap::new();
            map.insert(
                "com.apple.metadata:kMDItemFinderComment".to_string(),
                vec![116, 101, 115, 116, 32, 99, 111, 109, 109, 101, 110, 116],
            );
            map
        },
        security_context: None,
        platform_specific: PlatformMetadata::Unix(UnixMetadata {
            selinux_context: None,
            posix_acls: None,
        }),
    }
}
