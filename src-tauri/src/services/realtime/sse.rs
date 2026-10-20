/// One dispatched Server-Sent Event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SseMessage {
    pub event: Option<String>,
    pub data: String,
}

/// Incremental parser for the text/event-stream format; chunks may split lines anywhere.
#[derive(Default)]
pub struct SseParser {
    buffer: String,
    data: Vec<String>,
    event: Option<String>,
}

impl SseParser {
    pub fn push(&mut self, chunk: &str) -> Vec<SseMessage> {
        self.buffer.push_str(chunk);
        let mut out = Vec::new();
        while let Some(end) = self.buffer.find('\n') {
            let line: String = self.buffer.drain(..=end).collect();
            let line = line.trim_end_matches(['\n', '\r']);
            if line.is_empty() {
                if !self.data.is_empty() {
                    out.push(SseMessage {
                        event: self.event.take(),
                        data: self.data.join("\n"),
                    });
                    self.data.clear();
                }
                self.event = None;
                continue;
            }
            if line.starts_with(':') {
                continue;
            }
            let (field, value) = match line.split_once(':') {
                Some((f, v)) => (f, v.strip_prefix(' ').unwrap_or(v)),
                None => (line, ""),
            };
            match field {
                "data" => self.data.push(value.to_string()),
                "event" => self.event = Some(value.to_string()),
                _ => {}
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_events_across_chunk_boundaries() {
        let mut parser = SseParser::default();
        assert!(parser.push(": ping\nevent: tick\nda").is_empty());
        let messages = parser.push("ta: 1\r\ndata: 2\n\ndata: plain\n\n");
        assert_eq!(
            messages,
            vec![
                SseMessage {
                    event: Some("tick".into()),
                    data: "1\n2".into()
                },
                SseMessage {
                    event: None,
                    data: "plain".into()
                },
            ]
        );
    }

    #[test]
    fn ignores_blocks_without_data() {
        let mut parser = SseParser::default();
        assert!(parser.push("event: empty\nid: 3\n\n").is_empty());
        assert_eq!(parser.push("data:x\n\n")[0].event, None);
    }
}
