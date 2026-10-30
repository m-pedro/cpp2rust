#![forbid(unsafe_code)]

use std::fmt;

/// Zero-copy representation of a single parsed HTTP access log line.
/// All string fields borrow directly from the underlying memory-mapped file slice `'a`.
#[derive(Debug, Clone, PartialEq)]
pub struct LogRecord<'a> {
    pub ip: &'a str,
    pub timestamp: &'a str,
    pub method: &'a str,
    pub path: &'a str,
    pub status_code: u16,
    pub bytes_sent: u64,
    pub latency_ms: f64,
}

/// Structured error cases replacing the magic negative integers (-1..-11) in parser.cpp
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    EmptyLine,
    MissingIp,
    MissingTimestampOpen,
    MissingTimestampClose,
    MissingRequestQuoteOpen,
    MissingRequestQuoteClose,
    MalformedRequestLine,
    InvalidStatusCode,
    InvalidBytesSent,
    InvalidLatency,
    InvalidUtf8,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::EmptyLine => write!(f, "log line is empty"),
            ParseError::MissingIp => write!(f, "missing client IP field"),
            ParseError::MissingTimestampOpen => write!(f, "missing '[' before timestamp"),
            ParseError::MissingTimestampClose => write!(f, "missing ']' after timestamp"),
            ParseError::MissingRequestQuoteOpen => write!(f, "missing opening quote for HTTP request"),
            ParseError::MissingRequestQuoteClose => write!(f, "missing closing quote for HTTP request"),
            ParseError::MalformedRequestLine => write!(f, "malformed HTTP request method/path/version"),
            ParseError::InvalidStatusCode => write!(f, "HTTP status code out of valid range (100..=599)"),
            ParseError::InvalidBytesSent => write!(f, "failed to parse bytes_sent integer"),
            ParseError::InvalidLatency => write!(f, "failed to parse latency_ms float"),
            ParseError::InvalidUtf8 => write!(f, "field contains invalid UTF-8 bytes"),
        }
    }
}

impl std::error::Error for ParseError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_record_borrows_without_allocation() {
        let raw_line = String::from("10.0.0.1 GET /index.html");
        let record = LogRecord {
            ip: &raw_line[0..8],
            timestamp: "25/Oct/2026:00:00:01 +0000",
            method: &raw_line[9..12],
            path: &raw_line[13..24],
            status_code: 200,
            bytes_sent: 512,
            latency_ms: 1.25,
        };

        assert_eq!(record.ip, "10.0.0.1");
        assert_eq!(record.method, "GET");
        assert_eq!(record.path, "/index.html");
    }
}
