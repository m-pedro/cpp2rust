#![forbid(unsafe_code)]

use crate::types::{LogRecord, ParseError};
use std::str;

#[inline]
fn skip_spaces(mut bytes: &[u8]) -> &[u8] {
    while let Some((&b' ', rest)) = bytes.split_first() {
        bytes = rest;
    }
    bytes
}

#[inline]
fn find_byte(bytes: &[u8], target: u8) -> Option<usize> {
    bytes.iter().position(|&b| b == target)
}

#[inline]
fn parse_u64_fast(bytes: &[u8]) -> Option<u64> {
    if bytes.is_empty() {
        return None;
    }
    let mut val: u64 = 0;
    for &b in bytes {
        if !b.is_ascii_digit() {
            return None;
        }
        val = val.checked_mul(10)?.checked_add((b - b'0') as u64)?;
    }
    Some(val)
}

/// Parses a single HTTP access log line (`&'a [u8]`) into a zero-copy `LogRecord<'a>`.
///
/// Format:
/// `<ip> - - [<timestamp>] "<method> <path> <proto>" <status> <bytes> <latency_ms>`
pub fn parse_log_line_rs<'a>(line: &'a [u8]) -> Result<LogRecord<'a>, ParseError> {
    // Strip trailing '\r' or '\n' if present
    let mut rest = line;
    while let Some(&last) = rest.last() {
        if last == b'\n' || last == b'\r' {
            rest = &rest[..rest.len() - 1];
        } else {
            break;
        }
    }

    if rest.is_empty() {
        return Err(ParseError::EmptyLine);
    }

    // 1. Client IP up to first space
    let space_pos = find_byte(rest, b' ').ok_or(ParseError::MissingIp)?;
    if space_pos == 0 {
        return Err(ParseError::MissingIp);
    }
    let ip = str::from_utf8(&rest[..space_pos]).map_err(|_| ParseError::InvalidUtf8)?;
    rest = &rest[space_pos + 1..];

    // 2. Timestamp inside '[' and ']'
    let open_bracket = find_byte(rest, b'[').ok_or(ParseError::MissingTimestampOpen)?;
    rest = &rest[open_bracket + 1..];

    let close_bracket = find_byte(rest, b']').ok_or(ParseError::MissingTimestampClose)?;
    let timestamp = str::from_utf8(&rest[..close_bracket]).map_err(|_| ParseError::InvalidUtf8)?;
    rest = &rest[close_bracket + 1..];

    // 3. Quoted HTTP request: "<method> <path> <protocol>"
    let quote_open = find_byte(rest, b'"').ok_or(ParseError::MissingRequestQuoteOpen)?;
    rest = &rest[quote_open + 1..];

    let quote_close = find_byte(rest, b'"').ok_or(ParseError::MissingRequestQuoteClose)?;
    let req_slice = &rest[..quote_close];
    rest = &rest[quote_close + 1..];

    let req_space1 = find_byte(req_slice, b' ').ok_or(ParseError::MalformedRequestLine)?;
    let method = str::from_utf8(&req_slice[..req_space1]).map_err(|_| ParseError::InvalidUtf8)?;

    let after_method = &req_slice[req_space1 + 1..];
    let req_space2 = find_byte(after_method, b' ').ok_or(ParseError::MalformedRequestLine)?;
    let path = str::from_utf8(&after_method[..req_space2]).map_err(|_| ParseError::InvalidUtf8)?;

    // 4. HTTP Status Code (100..=599)
    rest = skip_spaces(rest);
    let status_end = find_byte(rest, b' ').ok_or(ParseError::InvalidStatusCode)?;
    let status_raw = parse_u64_fast(&rest[..status_end]).ok_or(ParseError::InvalidStatusCode)?;
    if !(100..=599).contains(&status_raw) {
        return Err(ParseError::InvalidStatusCode);
    }
    let status_code = status_raw as u16;
    rest = &rest[status_end + 1..];

    // 5. Bytes Sent
    rest = skip_spaces(rest);
    let bytes_end = find_byte(rest, b' ').ok_or(ParseError::InvalidBytesSent)?;
    let bytes_sent = parse_u64_fast(&rest[..bytes_end]).ok_or(ParseError::InvalidBytesSent)?;
    rest = &rest[bytes_end + 1..];

    // 6. Latency (ms)
    rest = skip_spaces(rest);
    // Trim any trailing spaces
    while let Some(&b' ') = rest.last() {
        rest = &rest[..rest.len() - 1];
    }
    if rest.is_empty() {
        return Err(ParseError::InvalidLatency);
    }
    let latency_str = str::from_utf8(rest).map_err(|_| ParseError::InvalidUtf8)?;
    let latency_ms: f64 = latency_str.parse().map_err(|_| ParseError::InvalidLatency)?;
    if latency_ms < 0.0 || !latency_ms.is_finite() {
        return Err(ParseError::InvalidLatency);
    }

    Ok(LogRecord {
        ip,
        timestamp,
        method,
        path,
        status_code,
        bytes_sent,
        latency_ms,
    })
}
