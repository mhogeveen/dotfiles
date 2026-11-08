/*
 * a small json reader, for answers like `tailscale status --json` and mango's
 * `watch all-monitors`; amane's own json library isn't available to the config,
 * which can't add crates
 *
 * reading is forgiving the way views like it: a missing key or the wrong kind
 * gives an empty answer (Null, "", false, an empty list) instead of an error
 */

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Number(f64),
    Text(String),
    List(Vec<Value>),

    // in the order the answer gave them
    Object(Vec<(String, Value)>),
}

static NULL: Value = Value::Null;

impl Value {
    // the value under this key, Null when there's no such key or this isn't an object
    pub fn get(&self, key: &str) -> &Value {
        let Value::Object(entries) = self else {
            return &NULL;
        };

        entries
            .iter()
            .find(|(name, _)| name == key)
            .map_or(&NULL, |(_, value)| value)
    }

    pub fn text(&self) -> &str {
        match self {
            Value::Text(text) => text,
            _ => "",
        }
    }

    // nothing reads these three since tailscale's peer list went, kept for whoever does
    #[allow(dead_code)]
    pub fn bool(&self) -> bool {
        matches!(self, Value::Bool(true))
    }

    // nothing reads numbers yet, the reader has them for whoever does
    #[allow(dead_code)]
    pub fn number(&self) -> Option<f64> {
        match self {
            Value::Number(number) => Some(*number),
            _ => None,
        }
    }

    pub fn list(&self) -> &[Value] {
        match self {
            Value::List(list) => list,
            _ => &[],
        }
    }

    #[allow(dead_code)]
    pub fn entries(&self) -> &[(String, Value)] {
        match self {
            Value::Object(entries) => entries,
            _ => &[],
        }
    }

    #[allow(dead_code)]
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }
}

// the whole text must be one json value, with only whitespace around it
pub fn parse(text: &str) -> Option<Value> {
    let mut reader = Reader {
        characters: text.chars().collect(),
        position: 0,
    };

    let value = reader.value()?;

    reader.skip_space();

    (reader.position == reader.characters.len()).then_some(value)
}

struct Reader {
    characters: Vec<char>,
    position: usize,
}

impl Reader {
    fn peek(&self) -> Option<char> {
        self.characters.get(self.position).copied()
    }

    fn next(&mut self) -> Option<char> {
        let character = self.peek()?;

        self.position += 1;

        Some(character)
    }

    fn skip_space(&mut self) {
        while self.peek().is_some_and(char::is_whitespace) {
            self.position += 1;
        }
    }

    fn expect(&mut self, word: &str) -> Option<()> {
        for expected in word.chars() {
            if self.next()? != expected {
                return None;
            }
        }

        Some(())
    }

    fn value(&mut self) -> Option<Value> {
        self.skip_space();

        match self.peek()? {
            '{' => self.object(),
            '[' => self.list(),
            '"' => self.text().map(Value::Text),
            't' => self.expect("true").map(|()| Value::Bool(true)),
            'f' => self.expect("false").map(|()| Value::Bool(false)),
            'n' => self.expect("null").map(|()| Value::Null),
            _ => self.number(),
        }
    }

    fn object(&mut self) -> Option<Value> {
        self.expect("{")?;

        let mut entries = Vec::new();

        self.skip_space();

        if self.peek() == Some('}') {
            self.position += 1;

            return Some(Value::Object(entries));
        }

        loop {
            self.skip_space();

            let key = self.text()?;

            self.skip_space();
            self.expect(":")?;

            let value = self.value()?;

            entries.push((key, value));

            self.skip_space();

            match self.next()? {
                ',' => continue,
                '}' => return Some(Value::Object(entries)),
                _ => return None,
            }
        }
    }

    fn list(&mut self) -> Option<Value> {
        self.expect("[")?;

        let mut items = Vec::new();

        self.skip_space();

        if self.peek() == Some(']') {
            self.position += 1;

            return Some(Value::List(items));
        }

        loop {
            items.push(self.value()?);

            self.skip_space();

            match self.next()? {
                ',' => continue,
                ']' => return Some(Value::List(items)),
                _ => return None,
            }
        }
    }

    // a quoted string, escapes undone
    fn text(&mut self) -> Option<String> {
        self.expect("\"")?;

        let mut text = String::new();

        loop {
            match self.next()? {
                '"' => return Some(text),

                '\\' => match self.next()? {
                    '"' => text.push('"'),
                    '\\' => text.push('\\'),
                    '/' => text.push('/'),
                    'n' => text.push('\n'),
                    't' => text.push('\t'),
                    'r' => text.push('\r'),
                    'b' => text.push('\u{8}'),
                    'f' => text.push('\u{c}'),
                    'u' => text.push(self.unicode()?),
                    _ => return None,
                },

                character => text.push(character),
            }
        }
    }

    // \uXXXX, with \uD83D\uDE00-style pairs joined into one character
    fn unicode(&mut self) -> Option<char> {
        let first = self.hex4()?;

        if !(0xD800..0xDC00).contains(&first) {
            return Some(char::from_u32(first).unwrap_or('\u{fffd}'));
        }

        // the second half of a surrogate pair should follow
        if self.peek() != Some('\\') {
            return Some('\u{fffd}');
        }

        self.position += 1;
        self.expect("u")?;

        let second = self.hex4()?;

        let code = 0x10000 + ((first - 0xD800) << 10) + (second.wrapping_sub(0xDC00) & 0x3FF);

        Some(char::from_u32(code).unwrap_or('\u{fffd}'))
    }

    fn hex4(&mut self) -> Option<u32> {
        let mut code = 0;

        for _ in 0..4 {
            code = code * 16 + self.next()?.to_digit(16)?;
        }

        Some(code)
    }

    fn number(&mut self) -> Option<Value> {
        let start = self.position;

        while self
            .peek()
            .is_some_and(|c| c.is_ascii_digit() || matches!(c, '-' | '+' | '.' | 'e' | 'E'))
        {
            self.position += 1;
        }

        let text: String = self.characters[start..self.position].iter().collect();

        text.parse().ok().map(Value::Number)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_nested_values() {
        let value = parse(
            r#" { "Self": { "HostName": "prometheus", "TailscaleIPs": ["100.1.2.3", "fd7a::1"],
                 "Online": true, "Peers": 3, "Gone": null }, "Empty": {}, "None": [] } "#,
        )
        .expect("json");

        let me = value.get("Self");

        assert_eq!(me.get("HostName").text(), "prometheus");
        assert_eq!(me.get("TailscaleIPs").list()[0].text(), "100.1.2.3");
        assert!(me.get("Online").bool());
        assert_eq!(me.get("Peers").number(), Some(3.0));
        assert!(me.get("Gone").is_null());
        assert!(value.get("Empty").entries().is_empty());
        assert!(value.get("None").list().is_empty());
    }

    #[test]
    fn missing_keys_and_wrong_kinds_read_as_empty() {
        let value = parse(r#"{"a": 1}"#).expect("json");

        assert!(value.get("b").is_null());
        assert_eq!(value.get("a").text(), "");
        assert!(!value.get("a").bool());
        assert!(value.get("a").get("deeper").is_null());
    }

    #[test]
    fn escapes_and_numbers() {
        let value = parse(r#"["a\"b\\c\/d\n", "\u00e9\uD83D\uDE00", -1.5e2, 0]"#).expect("json");

        assert_eq!(value.list()[0].text(), "a\"b\\c/d\n");
        assert_eq!(value.list()[1].text(), "é😀");
        assert_eq!(value.list()[2].number(), Some(-150.0));
        assert_eq!(value.list()[3].number(), Some(0.0));
    }

    #[test]
    fn broken_json_is_none() {
        assert!(parse("").is_none());
        assert!(parse("{").is_none());
        assert!(parse(r#"{"a" 1}"#).is_none());
        assert!(parse(r#"{"a": 1,}"#).is_none());
        assert!(parse("[1, 2] extra").is_none());
        assert!(parse("tru").is_none());
    }
}
