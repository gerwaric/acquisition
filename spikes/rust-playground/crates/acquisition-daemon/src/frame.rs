//! The frame reader both sides of the socket use (C85): one line, read
//! no further than [`MAX_FRAME_BYTES`](acquisition_protocol::protocol::MAX_FRAME_BYTES).
//! A line past the bound is
//! discarded through its newline and reported as [`Frame::Oversize`], so
//! the reader never buffers a runaway line and the next frame is still
//! aligned — the daemon answers `bad_request` and keeps the connection,
//! the client reports the answer it could not read and keeps its
//! connection too. Bytes, not text: a frame that is not UTF-8 is the JSON
//! parser's failure to report, like any other malformed frame.
//!
//! One copy here and one in `acquisition-client/src/frame.rs`, byte for
//! byte below this paragraph (the daemon split's step 3): the reader
//! needs tokio, the protocol crate is serde-only (C1; `tools/docs-check.sh`'s
//! allowlist refuses a tokio feature there), and the daemon never links
//! the client. The constant both copies read is the protocol's, so the
//! bound cannot drift; the tests below run in both.

use tokio::io::{AsyncBufRead, AsyncBufReadExt};

/// One read from a framed connection.
#[derive(Debug, PartialEq, Eq)]
pub enum Frame {
    /// A line, without its newline; possibly empty.
    Line(Vec<u8>),
    /// A line longer than the bound, discarded through its newline; the
    /// connection is aligned on the next frame.
    Oversize,
    /// The peer closed the connection. An oversize line cut off by the
    /// close reads as `Closed` too — there is nobody left to answer.
    Closed,
}

/// Read the next frame, buffering at most `max` bytes of it.
pub async fn read_frame<R: AsyncBufRead + Unpin>(
    reader: &mut R,
    max: usize,
) -> std::io::Result<Frame> {
    let mut line = Vec::new();
    let mut discarding = false;
    loop {
        let buf = reader.fill_buf().await?;
        if buf.is_empty() {
            return Ok(if discarding || line.is_empty() {
                Frame::Closed
            } else {
                Frame::Line(line)
            });
        }
        let newline = buf.iter().position(|&b| b == b'\n');
        let chunk = &buf[..newline.unwrap_or(buf.len())];
        if !discarding {
            if line.len() + chunk.len() > max {
                discarding = true;
                line = Vec::new();
            } else {
                line.extend_from_slice(chunk);
            }
        }
        let consumed = chunk.len() + usize::from(newline.is_some());
        reader.consume(consumed);
        if newline.is_some() {
            return Ok(if discarding {
                Frame::Oversize
            } else {
                Frame::Line(line)
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::BufReader;

    /// The bound is exact: a line of `max` bytes is a line, one more is
    /// oversize, and the frame after either is read intact.
    #[tokio::test]
    async fn a_line_over_the_bound_is_discarded_through_its_newline() {
        let max = 8;
        let input = b"12345678\n123456789\n{}\n\nlast".to_vec();
        let mut reader = BufReader::with_capacity(3, &input[..]);
        assert_eq!(
            read_frame(&mut reader, max).await.unwrap(),
            Frame::Line(b"12345678".to_vec())
        );
        assert_eq!(read_frame(&mut reader, max).await.unwrap(), Frame::Oversize);
        assert_eq!(
            read_frame(&mut reader, max).await.unwrap(),
            Frame::Line(b"{}".to_vec())
        );
        assert_eq!(
            read_frame(&mut reader, max).await.unwrap(),
            Frame::Line(Vec::new())
        );
        // An unterminated last line is still a line; then the close.
        assert_eq!(
            read_frame(&mut reader, max).await.unwrap(),
            Frame::Line(b"last".to_vec())
        );
        assert_eq!(read_frame(&mut reader, max).await.unwrap(), Frame::Closed);
    }

    /// An oversize line the peer never finished is a close, not a frame
    /// to answer.
    #[tokio::test]
    async fn an_oversize_line_cut_off_by_the_close_is_a_close() {
        let input = b"123456789".to_vec();
        let mut reader = BufReader::with_capacity(4, &input[..]);
        assert_eq!(read_frame(&mut reader, 8).await.unwrap(), Frame::Closed);
    }
}
