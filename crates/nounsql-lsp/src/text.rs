use tower_lsp::lsp_types::{Position, Range};

/// バイトオフセットと LSP の位置（UTF-16 単位）を相互変換する。
pub struct TextMap {
    line_starts: Vec<usize>,
    text: String,
}

impl TextMap {
    pub fn new(text: String) -> Self {
        let mut line_starts = vec![0];
        line_starts.extend(text.match_indices('\n').map(|(i, _)| i + 1));
        Self { line_starts, text }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn position(&self, offset: usize) -> Position {
        let offset = offset.min(self.text.len());
        let line = self.line_starts.partition_point(|&s| s <= offset).max(1) - 1;
        let start = self.line_starts[line];
        let utf16 = self.text[start..offset].encode_utf16().count();
        Position {
            line: line as u32,
            character: utf16 as u32,
        }
    }

    pub fn range(&self, start: usize, end: usize) -> Range {
        Range {
            start: self.position(start),
            end: self.position(end),
        }
    }

    pub fn offset(&self, pos: Position) -> usize {
        let Some(&line_start) = self.line_starts.get(pos.line as usize) else {
            return self.text.len();
        };
        let line_end = self
            .line_starts
            .get(pos.line as usize + 1)
            .copied()
            .unwrap_or(self.text.len());
        let line = &self.text[line_start..line_end];

        let mut utf16 = 0usize;
        for (i, ch) in line.char_indices() {
            if utf16 >= pos.character as usize {
                return line_start + i;
            }
            utf16 += ch.len_utf16();
        }
        line_end
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_multibyte() {
        let map =
            TextMap::new("table user {\n  column name type=text comment=\"名前\"\n}\n".into());
        let offset = map.text().find("名前").expect("found");
        let pos = map.position(offset);
        assert_eq!(map.offset(pos), offset);
    }
}
