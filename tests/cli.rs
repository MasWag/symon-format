// This file is derived from https://github.com/bbannier/spicy-format
// Original code is distributed under the MIT License.
// 
// MIT License
//
// Copyright (c) 2023 Benjamin Bannier 
// Copyright (c) 2025 Masaki Waga
// 
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
// 
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
// 
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use assert_cmd::cargo::CommandCargoExt;
use miette::miette;
use rayon::prelude::*;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Helper to format an input file via the CLI.
fn format<P>(path: P) -> Result<String>
where
    P: Into<PathBuf>,
{
    let path = path.into();
    let output = Command::cargo_bin("symon-format")?
        .arg("--reject-parse-errors")
        .arg(&path)
        .stdout(Stdio::piped())
        .output()?;

    assert!(output.status.success(), "could not format {path:?}");
    let output = String::from_utf8(output.stdout)?;

    Ok(output)
}

#[test]
fn corpus() -> Result<()> {
    use pretty_assertions::assert_eq;

    let corpus = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("corpus");

    let update_baseline = std::env::var("UPDATE_BASELINE").is_ok();

    let files = walkdir::WalkDir::new(&corpus)
        .into_iter()
        .filter_map(|e| {
            let e = e.ok()?;

            if e.file_type().is_file()
                && e.path().extension().and_then(|ext| ext.to_str()) == Some("symon")
            {
                Some(e)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    files
        .par_iter()
        .filter_map(|t| {
            let input = t.path();

            let output = {
                let path = input.to_path_buf();

                let file_name = format!(
                    "{}.expected",
                    path.file_name()
                        .unwrap_or_else(|| panic!(
                            "cannot get filename component of {}",
                            path.display()
                        ))
                        .to_string_lossy()
                );

                let mut o = path;
                assert!(o.pop());

                o.join(file_name)
            };

            let formatted = format(&input).unwrap_or_else(|_| {
                panic!("cannot format source file {}", t.path().to_string_lossy())
            });

            if !update_baseline {
                let expected = std::fs::read_to_string(output).expect("cannot read baseline");
                assert_eq!(
                    expected,
                    formatted,
                    "while formatting {}",
                    t.path().display()
                );
            } else {
                std::fs::write(output, formatted).expect("cannot update baseline");
            }

            Some(1)
        })
        .collect::<Vec<_>>();

    Ok(())
}

#[test]
fn corpus_external() -> miette::Result<()> {
    let Ok(corpus) = std::env::var("SYMON_FORMAT_EXTERNAL_CORPUS") else {
        return Ok(());
    };

    let is_filtered = |p: &Path| -> bool {
        let deny_list = [
            "tools/preprocessor.symon",
            "types/unit/hooks-fail.symon",
            // Unsupported legacy syntax.
            "types/vector/legacy-syntax-fail.symon",
            // Fails due to parser ambiguity due to https://github.com/zeek/symon/issues/1566.
            "types/unit/switch-attributes-fail.symon",
        ];

        deny_list.iter().any(|b| p.ends_with(b))
    };

    // Compute a vector of file names so we can process them below in parallel.
    let files = walkdir::WalkDir::new(&corpus)
        .into_iter()
        .filter_map(|e| {
            let e = e.ok()?;

            if e.file_type().is_file()
                && e.path().extension().and_then(|ext| ext.to_str()) == Some("symon")
                && !is_filtered(e.path())
            {
                Some(e)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let results = files
        .par_iter()
        .filter_map(|f| {
            let f = f.path().to_str()?;

            let source = std::fs::read_to_string(f).ok()?;

            // Ignore inputs with multiple parts.
            if source.contains("@TEST-START-FILE") {
                return None;
            }

            match format(&f) {
                Err(_) => Some((f.to_string(), false)),
                Ok(_) => Some((f.to_string(), true)),
            }
        })
        .collect::<HashMap<_, _>>();

    let num_tests = results.len();

    let failures = results
        .into_iter()
        .filter_map(|(f, success)| if success { None } else { Some(f) })
        .collect::<Vec<_>>();

    if failures.is_empty() {
        Ok(())
    } else {
        Err(miette!(
            "{} out of {num_tests} inputs failed:\n{}",
            failures.len(),
            failures.join("\n")
        ))
    }
}
