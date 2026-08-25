use crate::kernel::{KernelError, Result};

pub(crate) fn escape(value: &str) -> String {
    let mut output = String::new();
    for ch in value.chars() {
        match ch {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            ch if ch <= '\u{1f}' => output.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => output.push(ch),
        }
    }
    output
}

#[derive(Clone, Debug)]
pub(crate) enum Json {
    String(String),
    Array(Vec<Json>),
}

pub(crate) fn json(value: &Json) -> String {
    match value {
        Json::String(text) => format!("\"{}\"", escape(text)),
        Json::Array(values) => format!(
            "[{}]",
            values.iter().map(json).collect::<Vec<_>>().join(",")
        ),
    }
}

pub(crate) fn array<'a>(value: &'a Json, where_: &str) -> Result<&'a [Json]> {
    match value {
        Json::Array(values) => Ok(values),
        _ => Err(KernelError::new(format!("invalid {where_}"))),
    }
}

pub(crate) fn list<'a>(value: &'a Json, count: usize, where_: &str) -> Result<&'a [Json]> {
    let values = array(value, where_)?;
    if values.len() == count {
        Ok(values)
    } else {
        Err(KernelError::new(format!("invalid {where_}")))
    }
}

pub(crate) fn string<'a>(value: &'a Json, where_: &str) -> Result<&'a str> {
    match value {
        Json::String(text) => Ok(text),
        _ => Err(KernelError::new(format!("invalid {where_}"))),
    }
}

pub(crate) fn require_string(value: &Json, expected: &str, where_: &str) -> Result<()> {
    if string(value, where_)? == expected {
        Ok(())
    } else {
        Err(KernelError::new(format!("invalid {where_}")))
    }
}

pub(crate) struct JsonParser<'a> {
    input: &'a [u8],
    at: usize,
}

impl<'a> JsonParser<'a> {
    pub(crate) fn new(input: &'a str) -> Self {
        Self {
            input: input.as_bytes(),
            at: 0,
        }
    }

    pub(crate) fn parse(mut self) -> Result<Json> {
        let value = self.value()?;
        if self.at == self.input.len() {
            Ok(value)
        } else {
            Err(KernelError::new("trailing data in revision wire"))
        }
    }

    fn value(&mut self) -> Result<Json> {
        match self.peek() {
            Some(b'\"') => self.string().map(Json::String),
            Some(b'[') => self.array(),
            _ => Err(KernelError::new(
                "revision wire admits only arrays and strings",
            )),
        }
    }

    fn array(&mut self) -> Result<Json> {
        self.expect(b'[')?;
        let mut values = Vec::new();
        if self.peek() == Some(b']') {
            self.at += 1;
            return Ok(Json::Array(values));
        }
        loop {
            values.push(self.value()?);
            match self.peek() {
                Some(b',') => self.at += 1,
                Some(b']') => {
                    self.at += 1;
                    return Ok(Json::Array(values));
                }
                _ => return Err(KernelError::new("invalid JSON array")),
            }
        }
    }

    fn string(&mut self) -> Result<String> {
        self.expect(b'\"')?;
        let mut output = String::new();
        loop {
            let byte = self
                .next()
                .ok_or_else(|| KernelError::new("unterminated JSON string"))?;
            match byte {
                b'\"' => return Ok(output),
                b'\\' => match self
                    .next()
                    .ok_or_else(|| KernelError::new("truncated JSON escape"))?
                {
                    b'\"' => output.push('"'),
                    b'\\' => output.push('\\'),
                    b'n' => output.push('\n'),
                    b'r' => output.push('\r'),
                    b't' => output.push('\t'),
                    b'u' => {
                        let hex = self.take(4)?;
                        let text = std::str::from_utf8(hex)
                            .map_err(|_| KernelError::new("invalid JSON unicode escape"))?;
                        let scalar = u32::from_str_radix(text, 16)
                            .map_err(|_| KernelError::new("invalid JSON unicode escape"))?;
                        output.push(
                            char::from_u32(scalar)
                                .ok_or_else(|| KernelError::new("invalid JSON unicode escape"))?,
                        );
                    }
                    _ => return Err(KernelError::new("invalid JSON escape")),
                },
                0..=0x1f => return Err(KernelError::new("control character in JSON string")),
                _ if byte < 0x80 => output.push(byte as char),
                _ => {
                    let length = utf8_width(byte)
                        .ok_or_else(|| KernelError::new("invalid UTF-8 in JSON string"))?;
                    let mut encoded = vec![byte];
                    encoded.extend_from_slice(self.take(length - 1)?);
                    output.push_str(
                        std::str::from_utf8(&encoded)
                            .map_err(|_| KernelError::new("invalid UTF-8 in JSON string"))?,
                    );
                }
            }
        }
    }

    fn peek(&self) -> Option<u8> {
        self.input.get(self.at).copied()
    }

    fn next(&mut self) -> Option<u8> {
        let byte = self.peek()?;
        self.at += 1;
        Some(byte)
    }

    fn expect(&mut self, expected: u8) -> Result<()> {
        if self.next() == Some(expected) {
            Ok(())
        } else {
            Err(KernelError::new("invalid JSON"))
        }
    }

    fn take(&mut self, count: usize) -> Result<&'a [u8]> {
        let end = self
            .at
            .checked_add(count)
            .ok_or_else(|| KernelError::new("truncated JSON"))?;
        let result = self
            .input
            .get(self.at..end)
            .ok_or_else(|| KernelError::new("truncated JSON"))?;
        self.at = end;
        Ok(result)
    }
}

fn utf8_width(first: u8) -> Option<usize> {
    match first {
        0xc2..=0xdf => Some(2),
        0xe0..=0xef => Some(3),
        0xf0..=0xf4 => Some(4),
        _ => None,
    }
}
