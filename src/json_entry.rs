use crate::metadata::FileMetadata;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde::Serialize;
use std::path::Path;

// Struct for JSON serialization that handles attribute encoding
#[derive(Serialize)]
pub struct SerializableFileMetadata {
    pub path: String,
    pub size: u64,
    pub permissions: String,
    pub created: Option<String>,
    pub modified: Option<String>,
    pub accessed: Option<String>,
    pub extended_attributes: std::collections::HashMap<String, String>,
    pub security_context: Option<String>,
    pub platform_specific: crate::metadata::PlatformMetadata,
}

// Struct for JSON output
#[derive(Serialize)]
pub struct FileEntry {
    pub path: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<SerializableFileMetadata>,
}

impl FileEntry {
    pub fn new(
        path: &Path,
        content: String,
        metadata: Option<FileMetadata>,
        show_binary: bool,
    ) -> Self {
        let metadata = metadata.map(|meta| {
            let serializable_xattrs = meta
                .extended_attributes
                .into_iter()
                .filter_map(|(name, value)| {
                    if FileMetadata::is_text_attribute(&name, &value) {
                        Some((name, String::from_utf8_lossy(&value).to_string()))
                    } else if show_binary {
                        Some((name, STANDARD.encode(&value)))
                    } else {
                        None
                    }
                })
                .collect();

            SerializableFileMetadata {
                path: meta.path,
                size: meta.size,
                permissions: meta.permissions,
                created: meta.created,
                modified: meta.modified,
                accessed: meta.accessed,
                extended_attributes: serializable_xattrs,
                security_context: meta.security_context,
                platform_specific: meta.platform_specific,
            }
        });

        Self {
            path: path.display().to_string(),
            content,
            metadata,
        }
    }
}
