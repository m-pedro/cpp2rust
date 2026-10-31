use cpp2rust::parser::parse_log_line_rs;
use cpp2rust::types::ParseError;

#[test]
fn test_every_parse_error_variant() {
    assert_eq!(parse_log_line_rs(b""), Err(ParseError::EmptyLine));
    assert_eq!(parse_log_line_rs(b"\r\n"), Err(ParseError::EmptyLine));

    assert_eq!(parse_log_line_rs(b"10.0.0.1"), Err(ParseError::MissingIp));
    assert_eq!(
        parse_log_line_rs(b" - - [25/Oct/2026:00:00:01 +0000]"),
        Err(ParseError::MissingIp)
    );

    assert_eq!(
        parse_log_line_rs(b"10.0.0.1 - - 25/Oct/2026:00:00:01 +0000]"),
        Err(ParseError::MissingTimestampOpen)
    );
    assert_eq!(
        parse_log_line_rs(b"10.0.0.1 - - [25/Oct/2026:00:00:01 +0000"),
        Err(ParseError::MissingTimestampClose)
    );

    assert_eq!(
        parse_log_line_rs(b"10.0.0.1 - - [25/Oct/2026:00:00:01 +0000] GET / HTTP/1.1"),
        Err(ParseError::MissingRequestQuoteOpen)
    );
    assert_eq!(
        parse_log_line_rs(b"10.0.0.1 - - [25/Oct/2026:00:00:01 +0000] \"GET / HTTP/1.1"),
        Err(ParseError::MissingRequestQuoteClose)
    );
    assert_eq!(
        parse_log_line_rs(b"10.0.0.1 - - [25/Oct/2026:00:00:01 +0000] \"GET_ONLY_ONE_TOKEN\" 200 10 1.0"),
        Err(ParseError::MalformedRequestLine)
    );

    // Status code out of range (99 and 600)
    assert_eq!(
        parse_log_line_rs(b"10.0.0.1 - - [25/Oct/2026:00:00:01 +0000] \"GET / HTTP/1.1\" 99 100 1.0"),
        Err(ParseError::InvalidStatusCode)
    );
    assert_eq!(
        parse_log_line_rs(b"10.0.0.1 - - [25/Oct/2026:00:00:01 +0000] \"GET / HTTP/1.1\" 600 100 1.0"),
        Err(ParseError::InvalidStatusCode)
    );

    // u64 integer overflow on bytes_sent (18446744073709551616 == u64::MAX + 1)
    assert_eq!(
        parse_log_line_rs(
            b"10.0.0.1 - - [25/Oct/2026:00:00:01 +0000] \"GET / HTTP/1.1\" 200 18446744073709551616 1.0"
        ),
        Err(ParseError::InvalidBytesSent)
    );

    // Negative, NaN, and Inf latencies must be rejected
    assert_eq!(
        parse_log_line_rs(b"10.0.0.1 - - [25/Oct/2026:00:00:01 +0000] \"GET / HTTP/1.1\" 200 100 -5.0"),
        Err(ParseError::InvalidLatency)
    );
    assert_eq!(
        parse_log_line_rs(b"10.0.0.1 - - [25/Oct/2026:00:00:01 +0000] \"GET / HTTP/1.1\" 200 100 NaN"),
        Err(ParseError::InvalidLatency)
    );
    assert_eq!(
        parse_log_line_rs(b"10.0.0.1 - - [25/Oct/2026:00:00:01 +0000] \"GET / HTTP/1.1\" 200 100 inf"),
        Err(ParseError::InvalidLatency)
    );

    // Invalid UTF-8 byte sequence (0xFF) inside IP address
    assert_eq!(
        parse_log_line_rs(b"\xFF\xFE - - [25/Oct/2026:00:00:01 +0000] \"GET / HTTP/1.1\" 200 100 1.0"),
        Err(ParseError::InvalidUtf8)
    );
}

#[test]
fn test_arbitrary_corrupted_slices_never_panic() {
    let base = b"192.168.1.10 - - [25/Oct/2026:12:00:00 +0000] \"POST /api/v1/pay HTTP/1.1\" 200 512 12.34";
    // Every possible prefix truncation of a valid log line must return Err without panicking
    for cut in 0..base.len() {
        let _ = parse_log_line_rs(&base[..cut]);
    }
}
