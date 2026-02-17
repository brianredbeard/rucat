//! Cross-platform metadata support for rucat
//!
//! This module provides safe access to platform-specific file metadata
//! including extended attributes, `SELinux` contexts, and Windows alternate data streams.
use crate::binary_display::{BinaryDataFormatter, BinaryFormatOptions};
use serde::Serialize;
use std::collections::HashMap;
use std::fmt::Write;
use std::io;
use std::path::Path;

#[cfg(target_family = "unix")]
const MAX_XATTR_DISPLAY_SIZE: usize = 1_048_576; // 1 MiB

/// Represents cross-platform file metadata that can be displayed
#[derive(Debug, Clone, Serialize)]
pub struct FileMetadata {
    pub path: String,
    pub size: u64,
    pub permissions: String,
    pub created: Option<String>,
    pub modified: Option<String>,
    pub accessed: Option<String>,
    pub extended_attributes: HashMap<String, Vec<u8>>,
    pub security_context: Option<String>,
    pub platform_specific: PlatformMetadata,
}

#[cfg(unix)]
fn format_permissions(meta: &std::fs::Metadata) -> String {
    use std::os::unix::fs::PermissionsExt;
    let mode = meta.permissions().mode();
    let file_type = if meta.is_dir() {
        'd'
    } else if meta.is_symlink() {
        'l'
    } else {
        '-'
    };
    format!(
        "{}{}{}{}{}{}{}{}{}{}",
        file_type,
        if mode & 0o400 != 0 { 'r' } else { '-' }, // user_r
        if mode & 0o200 != 0 { 'w' } else { '-' }, // user_w
        if mode & 0o100 != 0 { 'x' } else { '-' }, // user_x
        if mode & 0o040 != 0 { 'r' } else { '-' }, // group_r
        if mode & 0o020 != 0 { 'w' } else { '-' }, // group_w
        if mode & 0o010 != 0 { 'x' } else { '-' }, // group_x
        if mode & 0o004 != 0 { 'r' } else { '-' }, // other_r
        if mode & 0o002 != 0 { 'w' } else { '-' }, // other_w
        if mode & 0o001 != 0 { 'x' } else { '-' }, // other_x
    )
}

#[cfg(not(unix))]
fn format_permissions(meta: &std::fs::Metadata) -> String {
    if meta.permissions().readonly() {
        "readonly".to_string()
    } else {
        "writable".to_string()
    }
}

fn format_system_time(st: std::time::SystemTime) -> String {
    use time::{OffsetDateTime, UtcOffset, format_description};

    let dt: OffsetDateTime = st.into();
    let local_offset = UtcOffset::current_local_offset().unwrap_or(UtcOffset::UTC);
    let dt_local = dt.to_offset(local_offset);
    // Format: "Aug 20 04:24:00 2025"
    let format =
        format_description::parse("[month repr:short] [day] [hour]:[minute]:[second] [year]")
            .unwrap();
    dt_local
        .format(&format)
        .unwrap_or_else(|_| "Invalid date".to_string())
}

#[derive(Debug, Clone, Serialize)]
pub enum PlatformMetadata {
    Unix(UnixMetadata),
    Windows(WindowsMetadata),
    Unsupported,
}

#[derive(Debug, Clone, Serialize)]
pub struct UnixMetadata {
    pub selinux_context: Option<String>,
    pub posix_acls: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct WindowsMetadata {
    pub alternate_data_streams: Vec<String>,
    pub security_descriptor: Option<String>,
    pub file_attributes: u32,
}

impl FileMetadata {
    /// Extract metadata for a given file path
    ///
    /// # Errors
    ///
    /// Returns an error if the file's metadata or extended attributes
    /// cannot be accessed.
    pub fn extract<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        let fs_meta = std::fs::metadata(path.as_ref())?;

        let mut metadata = Self {
            path: path_str,
            size: fs_meta.len(),
            permissions: format_permissions(&fs_meta),
            created: fs_meta.created().ok().map(format_system_time),
            modified: fs_meta.modified().ok().map(format_system_time),
            accessed: fs_meta.accessed().ok().map(format_system_time),
            extended_attributes: HashMap::new(),
            security_context: None,
            platform_specific: PlatformMetadata::Unsupported,
        };

        // Extract platform-specific metadata
        #[cfg(target_family = "unix")]
        {
            metadata.platform_specific =
                PlatformMetadata::Unix(Self::extract_unix_metadata(&path)?);
            metadata.extended_attributes = Self::extract_unix_xattrs(&path)?;
            metadata.security_context = Self::extract_selinux_context(&path)?;
        }

        #[cfg(target_family = "windows")]
        {
            metadata.platform_specific =
                PlatformMetadata::Windows(Self::extract_windows_metadata(&path)?);
        }

        Ok(metadata)
    }

    #[cfg(target_family = "unix")]
    fn extract_unix_metadata<P: AsRef<Path>>(path: P) -> io::Result<UnixMetadata> {
        Ok(UnixMetadata {
            selinux_context: Self::extract_selinux_context(path.as_ref())?,
            posix_acls: Self::extract_posix_acls(path.as_ref()),
        })
    }

    #[cfg(target_family = "unix")]
    fn extract_unix_xattrs<P: AsRef<Path>>(path: P) -> io::Result<HashMap<String, Vec<u8>>> {
        let mut xattrs = HashMap::new();
        // The list function returns an empty iterator if xattrs are not supported by the filesystem,
        // which is the desired behavior.
        for attr_name in xattr::list(&path)? {
            // It's possible for an attribute to be set with a non-UTF8 name.
            // We'll ignore such attributes as they are rare and not displayable as text.
            if let Some(name_str) = attr_name.to_str() {
                // If an attribute is present in the list, its value should be gettable.
                // We use Ok(...) to proceed, treating an error on a single attribute as non-fatal
                // for the whole metadata extraction. `xattr::get` returns Ok(None)` if the value is gone
                // between list and get, which we ignore.
                if let Ok(Some(value)) = xattr::get(&path, &attr_name) {
                    if value.len() > MAX_XATTR_DISPLAY_SIZE {
                        let mut truncated_value = value[..MAX_XATTR_DISPLAY_SIZE].to_vec();
                        truncated_value.extend_from_slice(b"... (truncated)");
                        xattrs.insert(name_str.to_string(), truncated_value);
                    } else {
                        xattrs.insert(name_str.to_string(), value);
                    }
                }
            }
        }
        Ok(xattrs)
    }

    #[cfg(target_family = "unix")]
    fn extract_selinux_context<P: AsRef<Path>>(path: P) -> io::Result<Option<String>> {
        // xattr::get returns Ok(None) if attribute does not exist.
        // On some filesystems, it returns an error if not supported; we treat that as no context.
        match xattr::get(path, "security.selinux") {
            Ok(Some(mut context_buf)) => {
                // The value of security.selinux often includes a null terminator.
                if context_buf.last() == Some(&0) {
                    context_buf.pop();
                }

                String::from_utf8(context_buf).map(Some).map_err(|_| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Invalid UTF-8 in SELinux context",
                    )
                })
            }
            Ok(None) => Ok(None),
            Err(e) if e.kind() == io::ErrorKind::Unsupported => Ok(None),
            Err(e) => Err(e),
        }
    }

    #[cfg(target_family = "unix")]
    fn extract_posix_acls<P: AsRef<Path>>(path: P) -> Option<String> {
        // exacl::getfacl returns an error if the ACL is not present or not supported.
        match exacl::getfacl(path.as_ref(), None) {
            Ok(acls) if !acls.is_empty() => {
                let acl_str = acls
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<String>>()
                    .join("\n");
                Some(acl_str)
            }
            _ => None,
        }
    }

    #[cfg(target_family = "windows")]
    fn extract_windows_metadata<P: AsRef<Path>>(path: P) -> io::Result<WindowsMetadata> {
        use std::os::windows::ffi::OsStrExt;
        use windows_sys::Win32::Foundation::{ERROR_SUCCESS, INVALID_HANDLE_VALUE, LocalFree};
        use windows_sys::Win32::Security::Authorization::{
            ConvertSecurityDescriptorToStringSecurityDescriptorW, GetNamedSecurityInfoW,
            SE_FILE_OBJECT,
        };
        use windows_sys::Win32::Security::{
            DACL_SECURITY_INFORMATION, GROUP_SECURITY_INFORMATION, LABEL_SECURITY_INFORMATION,
            OWNER_SECURITY_INFORMATION, SACL_SECURITY_INFORMATION,
        };
        use windows_sys::Win32::Storage::FileSystem::{
            FindClose, FindFirstStreamW, FindNextStreamW, GetFileAttributesW,
            WIN32_FIND_STREAM_DATA,
        };

        let path_wide: Vec<u16> = path
            .as_ref()
            .as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        // Get file attributes
        let file_attributes = unsafe { GetFileAttributesW(path_wide.as_ptr()) };

        // Get Alternate Data Streams
        let mut alternate_data_streams = Vec::new();
        let mut stream_data: WIN32_FIND_STREAM_DATA = unsafe { std::mem::zeroed() };
        let find_handle = unsafe {
            FindFirstStreamW(
                path_wide.as_ptr(),
                0, // InfoLevel: FindStreamInfoStandard
                &mut stream_data as *mut _ as *mut _,
                0, // dwFlags
            )
        };

        if find_handle != INVALID_HANDLE_VALUE {
            loop {
                let len = (0..)
                    .take_while(|&i| stream_data.cStreamName[i] != 0)
                    .count();
                let stream_name_slice = &stream_data.cStreamName[0..len];
                let stream_name = String::from_utf16_lossy(stream_name_slice);

                // First stream is `::$DATA` (main content), which we skip.
                if !stream_name.is_empty() && stream_name != "::$DATA" {
                    let clean_name = stream_name
                        .strip_prefix(':')
                        .and_then(|s| s.strip_suffix(":$DATA"))
                        .unwrap_or(&stream_name);
                    alternate_data_streams.push(clean_name.to_string());
                }

                if unsafe { FindNextStreamW(find_handle, &mut stream_data as *mut _ as *mut _) }
                    == 0
                {
                    break;
                }
            }
            unsafe { FindClose(find_handle) };
        }

        // Get Security Descriptor as an SDDL string
        let security_descriptor = unsafe {
            let mut security_info = std::ptr::null_mut();
            let result = GetNamedSecurityInfoW(
                path_wide.as_ptr(),
                SE_FILE_OBJECT,
                DACL_SECURITY_INFORMATION
                    | GROUP_SECURITY_INFORMATION
                    | LABEL_SECURITY_INFORMATION
                    | OWNER_SECURITY_INFORMATION
                    | SACL_SECURITY_INFORMATION,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                &mut security_info,
            );

            if result == ERROR_SUCCESS {
                let mut sddl_ptr: *mut u16 = std::ptr::null_mut();
                let sddl = if ConvertSecurityDescriptorToStringSecurityDescriptorW(
                    security_info,
                    1, // SDDL_REVISION_1
                    DACL_SECURITY_INFORMATION
                        | GROUP_SECURITY_INFORMATION
                        | LABEL_SECURITY_INFORMATION
                        | OWNER_SECURITY_INFORMATION
                        | SACL_SECURITY_INFORMATION,
                    &mut sddl_ptr,
                    std::ptr::null_mut(),
                ) != 0
                {
                    let len = (0..).take_while(|&i| *sddl_ptr.add(i) != 0).count();
                    let sddl_slice = std::slice::from_raw_parts(sddl_ptr, len);
                    let sddl_string = String::from_utf16_lossy(sddl_slice);
                    LocalFree(sddl_ptr.cast());
                    Some(sddl_string)
                } else {
                    None
                };
                LocalFree(security_info);
                sddl
            } else {
                None
            }
        };

        Ok(WindowsMetadata {
            alternate_data_streams,
            security_descriptor,
            file_attributes,
        })
    }

    /// Format metadata for display
    #[must_use]
    pub fn format_for_display(&self, options: &BinaryFormatOptions) -> String {
        let mut output = String::new();

        // Basic file information
        writeln!(&mut output, "File: {}", self.path).unwrap();
        writeln!(&mut output, "Size: {} bytes", self.size).unwrap();
        writeln!(&mut output, "Permissions: {}", self.permissions).unwrap();

        if let Some(created) = &self.created {
            writeln!(&mut output, "Created: {created}").unwrap();
        }
        if let Some(modified) = &self.modified {
            writeln!(&mut output, "Modified: {modified}").unwrap();
        }
        if let Some(accessed) = &self.accessed {
            writeln!(&mut output, "Accessed: {accessed}").unwrap();
        }

        // Security context
        if let Some(ref context) = self.security_context {
            writeln!(&mut output, "SELinux Context: {context}").unwrap();
        }

        // Extended attributes
        if !self.extended_attributes.is_empty() {
            output.push_str("Extended Attributes:\n");
            for (name, value) in &self.extended_attributes {
                if Self::is_text_attribute(name, value) {
                    let value_str = String::from_utf8_lossy(value);
                    writeln!(&mut output, "  {name}: {value_str}").unwrap();
                } else if options.show_binary {
                    let formatter = BinaryDataFormatter::new(value, name);
                    let formatted_binary =
                        formatter.format(options.format, options.hex_width, options.base64_width);
                    // The binary output is a block, so we print the attribute name
                    // as a header and then the indented block.
                    writeln!(&mut output, "  {name}:").unwrap();
                    for line in formatted_binary.lines() {
                        writeln!(&mut output, "    {line}").unwrap();
                    }
                }
            }
        }

        // Platform-specific metadata
        match &self.platform_specific {
            PlatformMetadata::Unix(unix_meta) => {
                if let Some(acls) = &unix_meta.posix_acls {
                    output.push_str("POSIX ACLs:\n");
                    for acl in acls.lines() {
                        writeln!(&mut output, "  {acl}").unwrap();
                    }
                }
            }
            PlatformMetadata::Windows(win_meta) => {
                if !win_meta.alternate_data_streams.is_empty() {
                    output.push_str("Alternate Data Streams:\n");
                    for stream in &win_meta.alternate_data_streams {
                        writeln!(&mut output, "  {stream}").unwrap();
                    }
                }
                if let Some(sd) = &win_meta.security_descriptor {
                    writeln!(&mut output, "Security Descriptor (SDDL): {sd}").unwrap();
                }
                if win_meta.file_attributes != 0 {
                    #[cfg(target_family = "windows")]
                    {
                        let attributes_str = format_windows_attributes(win_meta.file_attributes);
                        writeln!(&mut output, "Attributes: {attributes_str}").unwrap();
                    }
                }
            }
            PlatformMetadata::Unsupported => {
                // Do not print anything if unsupported
            }
        }

        output
    }

    pub fn is_text_attribute(name: &str, value: &[u8]) -> bool {
        // Names known to be binary formats, even if they sometimes contain UTF-8
        if name.starts_with("com.apple.quarantine") || name.starts_with("com.apple.ResourceFork") {
            return false;
        }

        // SELinux contexts are always text
        if name.starts_with("security.selinux") {
            return true;
        }

        // The best heuristic is to check for valid UTF-8 and a low ratio of control characters.
        if std::str::from_utf8(value).is_err() {
            return false;
        }

        if value.is_empty() {
            return true; // Empty is considered text-like.
        }

        let control_chars = value
            .iter()
            .filter(|&&byte| {
                // C0 controls, except for HT, LF, FF, CR. Also check for DEL.
                (byte < 0x20 && ![9, 10, 12, 13].contains(&byte)) || byte == 0x7f
            })
            .count();

        // If less than 10% of characters are non-standard control characters, treat as text.
        let control_ratio = control_chars as f32 / value.len() as f32;
        control_ratio < 0.1
    }
}

#[cfg(target_family = "windows")]
fn format_windows_attributes(attributes: u32) -> String {
    use windows_sys::Win32::Storage::FileSystem::{
        FILE_ATTRIBUTE_ARCHIVE, FILE_ATTRIBUTE_COMPRESSED, FILE_ATTRIBUTE_DEVICE,
        FILE_ATTRIBUTE_DIRECTORY, FILE_ATTRIBUTE_ENCRYPTED, FILE_ATTRIBUTE_HIDDEN,
        FILE_ATTRIBUTE_INTEGRITY_STREAM, FILE_ATTRIBUTE_NO_SCRUB_DATA, FILE_ATTRIBUTE_NORMAL,
        FILE_ATTRIBUTE_NOT_CONTENT_INDEXED, FILE_ATTRIBUTE_OFFLINE, FILE_ATTRIBUTE_READONLY,
        FILE_ATTRIBUTE_RECALL_ON_DATA_ACCESS, FILE_ATTRIBUTE_RECALL_ON_OPEN,
        FILE_ATTRIBUTE_REPARSE_POINT, FILE_ATTRIBUTE_SPARSE_FILE, FILE_ATTRIBUTE_SYSTEM,
        FILE_ATTRIBUTE_TEMPORARY, FILE_ATTRIBUTE_VIRTUAL,
    };
    let mut parts = Vec::new();
    if attributes & FILE_ATTRIBUTE_READONLY != 0 {
        parts.push("READONLY");
    }
    if attributes & FILE_ATTRIBUTE_HIDDEN != 0 {
        parts.push("HIDDEN");
    }
    if attributes & FILE_ATTRIBUTE_SYSTEM != 0 {
        parts.push("SYSTEM");
    }
    if attributes & FILE_ATTRIBUTE_DIRECTORY != 0 {
        parts.push("DIRECTORY");
    }
    if attributes & FILE_ATTRIBUTE_ARCHIVE != 0 {
        parts.push("ARCHIVE");
    }
    if attributes & FILE_ATTRIBUTE_DEVICE != 0 {
        parts.push("DEVICE");
    }
    if attributes & FILE_ATTRIBUTE_NORMAL != 0 {
        parts.push("NORMAL");
    }
    if attributes & FILE_ATTRIBUTE_TEMPORARY != 0 {
        parts.push("TEMPORARY");
    }
    if attributes & FILE_ATTRIBUTE_SPARSE_FILE != 0 {
        parts.push("SPARSE_FILE");
    }
    if attributes & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
        parts.push("REPARSE_POINT");
    }
    if attributes & FILE_ATTRIBUTE_COMPRESSED != 0 {
        parts.push("COMPRESSED");
    }
    if attributes & FILE_ATTRIBUTE_OFFLINE != 0 {
        parts.push("OFFLINE");
    }
    if attributes & FILE_ATTRIBUTE_NOT_CONTENT_INDEXED != 0 {
        parts.push("NOT_CONTENT_INDEXED");
    }
    if attributes & FILE_ATTRIBUTE_ENCRYPTED != 0 {
        parts.push("ENCRYPTED");
    }
    if attributes & FILE_ATTRIBUTE_INTEGRITY_STREAM != 0 {
        parts.push("INTEGRITY_STREAM");
    }
    if attributes & FILE_ATTRIBUTE_VIRTUAL != 0 {
        parts.push("VIRTUAL");
    }
    if attributes & FILE_ATTRIBUTE_NO_SCRUB_DATA != 0 {
        parts.push("NO_SCRUB_DATA");
    }
    if attributes & FILE_ATTRIBUTE_RECALL_ON_OPEN != 0 {
        parts.push("RECALL_ON_OPEN");
    }
    if attributes & FILE_ATTRIBUTE_RECALL_ON_DATA_ACCESS != 0 {
        parts.push("RECALL_ON_DATA_ACCESS");
    }

    if parts.is_empty() {
        "None".to_string()
    } else {
        parts.join(" | ")
    }
}
