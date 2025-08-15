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
use rucat::formatters::{markdown::Markdown, Formatter};
use std::path::Path;

#[test]
fn md_basic() {
    let mut buf = Vec::new();
    let fmt = Markdown {
        line_numbers: false,
    }; // instantiate struct
    fmt.write(Path::new("foo.rs"), "fn main(){}", &mut buf)
        .unwrap();
    let out = String::from_utf8(buf).unwrap();
    assert!(out.contains("```rs"));
}
