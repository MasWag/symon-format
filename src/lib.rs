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

use {
    miette::{Diagnostic, Result, SourceOffset, SourceSpan},
    std::string::FromUtf8Error,
    thiserror::Error,
    topiary_core::{FormatterError, Operation, TopiaryQuery},
};

#[derive(Error, Debug, Diagnostic)]
#[error("format error")]
pub enum FormatError {
    #[diagnostic(code(symon_format::parse_error))]
    #[error("parse error")]
    Parse {
        #[source_code]
        src: String,

        #[label("syntax not understood")]
        err_span: SourceSpan,
    },

    #[error("internal query error")]
    Query(#[help] String),

    #[error("idempotency violated")]
    Idempotency,

    #[error("UTF8 conversion error")]
    UTF8(#[from] FromUtf8Error),

    #[error("unknown error")]
    Unknown,
}

const QUERY: &str = include_str!("query.scm");

/// Format SyMon source code.
///
/// # Arguments
///
/// - `input`: SyMon source code to format
/// - `tolerate_parsing_errors`: whether source code with syntax errors should be accepted or
///   rejected.
/// - `skip_idempotence`: skip check that AST of formatted source is identical to input. This is
///   intended for working around current formatter limitations.
///
/// # Examples
///
/// ```
/// # use symon_format::format;
/// let source = "signature foo {} foo();foo()";
/// assert_eq!(
///     format(source, false, false).unwrap(),
///     "signature foo {
/// }
/// foo( );
/// foo( )"
/// );
/// ```
pub fn format(
    input: &str,
    skip_idempotence: bool,
    tolerate_parsing_errors: bool,
) -> Result<String> {
    let mut output = Vec::new();

    let grammar = topiary_tree_sitter_facade::Language::from(tree_sitter_symon::LANGUAGE);

    let query = TopiaryQuery::new(&grammar, QUERY).map_err(|e| match e {
        FormatterError::Query(m, e) => FormatError::Query(match e {
            None => m,
            Some(e) => format!("{m}: {e}"),
        }),
        _ => FormatError::Unknown,
    })?;

    let language = {
        topiary_core::Language {
            name: "spicy".to_string(),
            indent: Some("    ".to_string()),
            grammar,
            query,
        }
    };

    if let Err(e) = topiary_core::formatter(
        &mut input.as_bytes(),
        &mut output,
        &language,
        Operation::Format {
            skip_idempotence,
            tolerate_parsing_errors,
        },
    ) {
        Err(match e {
            FormatterError::Query(m, e) => FormatError::Query(match e {
                None => m,
                Some(e) => format!("{m}: {e}"),
            }),
            FormatterError::Idempotence => FormatError::Idempotency,
            FormatterError::Parsing {
                start_line,
                start_column,
                end_line,
                end_column,
            } => {
                let start = SourceOffset::from_location(
                    input,
                    start_line
                        .try_into()
                        .expect("cannot represent u32 as usize"),
                    start_column
                        .try_into()
                        .expect("cannot represent u32 as usize"),
                );
                let end = SourceOffset::from_location(
                    input,
                    end_line.try_into().expect("cannot represent u32 as usize"),
                    end_column
                        .try_into()
                        .expect("cannot represent u32 as usize"),
                );
                FormatError::Parse {
                    src: input.to_string(),
                    err_span: (start.offset(), end.offset() - start.offset()).into(),
                }
            }
            _ => FormatError::Unknown,
        })?;
    }

    let output = String::from_utf8(output).map_err(FormatError::UTF8)?;

    // Final cleanup of result. If we received an input not ending in a newline, also return an
    // output without newline. We do not want to force a newline since we e.g., could be formatting
    // input received from an editor and do not want to insert additional newlines.
    if input.ends_with('\n') {
        Ok(output)
    } else {
        Ok(output.trim_end().into())
    }
}
