#![allow(clippy::multiple_crate_versions)]
// This file is part of rucat.
//
// rucat is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// rucat is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with rucat.  If not, see <https://www.gnu.org/licenses/>.
//
// Copyright (C) 2024 Brian 'redbeard' Harrington
use clap::Parser;
use rucat::metadata::{FileMetadata, PlatformMetadata, UnixMetadata, WindowsMetadata};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// File to generate a fixture from
    #[arg(required = true)]
    file: PathBuf,
}

fn main() {
    let args = Args::parse();
    match FileMetadata::extract(&args.file) {
        Ok(metadata) => print_as_rust_code(&metadata),
        Err(e) => eprintln!(
            "Error extracting metadata for {}: {}",
            args.file.display(),
            e
        ),
    }
}

fn print_as_rust_code(meta: &FileMetadata) {
    println!("FileMetadata {{");
    println!("    path: \"{}\".to_string(),", meta.path);
    println!("    size: {},", meta.size);
    println!("    permissions: \"{}\".to_string(),", meta.permissions);
    println!("    created: {:?},", meta.created.as_deref());
    println!("    modified: {:?},", meta.modified.as_deref());
    println!("    accessed: {:?},", meta.accessed.as_deref());
    print_hashmap_code(&meta.extended_attributes);
    println!(
        "    security_context: {:?},",
        meta.security_context.as_ref()
    );
    print_platform_specific_code(&meta.platform_specific);
    println!("}}");
}

fn print_hashmap_code(map: &HashMap<String, Vec<u8>>) {
    println!("    extended_attributes: {{");
    println!("        let mut map = std::collections::HashMap::new();");
    for (key, value) in map {
        println!(
            "        map.insert(\"{}\".to_string(), vec!{:?});",
            key,
            value.as_slice()
        );
    }
    println!("        map");
    println!("    }},");
}

fn print_platform_specific_code(ps: &PlatformMetadata) {
    match ps {
        PlatformMetadata::Unix(unix_meta) => print_unix_metadata_code(unix_meta),
        PlatformMetadata::Windows(win_meta) => print_windows_metadata_code(win_meta),
        PlatformMetadata::Unsupported => {
            println!("    platform_specific: PlatformMetadata::Unsupported,");
        }
    }
}

fn print_unix_metadata_code(meta: &UnixMetadata) {
    println!("    platform_specific: PlatformMetadata::Unix(UnixMetadata {{");
    println!("        selinux_context: {:?},", meta.selinux_context);
    println!("        posix_acls: {:?},", meta.posix_acls);
    println!("    }}),");
}

fn print_windows_metadata_code(meta: &WindowsMetadata) {
    println!("    platform_specific: PlatformMetadata::Windows(WindowsMetadata {{");
    println!(
        "        alternate_data_streams: {:?},",
        meta.alternate_data_streams
    );
    println!(
        "        security_descriptor: {:?},",
        meta.security_descriptor
    );
    println!("        file_attributes: {},", meta.file_attributes);
    println!("    }}),");
}
