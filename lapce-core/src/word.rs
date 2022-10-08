use xi_rope::{Cursor, Rope, RopeInfo};

/// Describe char classifications used to compose word boundaries
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum CharClassification {
    /// Carriage Return (`r`)
    Cr,
    /// Line feed (`\n`)
    Lf,
    /// Whitespace character
    Space,
    /// Any punctuation character
    Punctuation,
    /// Includes letters and all of non-ascii unicode
    Other,
}

/// A word boundary can be the start of a word, its end or both for punctuation
#[derive(PartialEq, Eq)]
enum WordBoundary {
    /// Denote that this is not a boundary
    Interior,
    /// A boundary indicating the end of a word
    Start,
    /// A boundary indicating the start of a word
    End,
    /// Both start and end boundaries (ex: punctuation characters)
    Both,
}

impl WordBoundary {
    fn is_start(&self) -> bool {
        *self == WordBoundary::Start || *self == WordBoundary::Both
    }

    fn is_end(&self) -> bool {
        *self == WordBoundary::End || *self == WordBoundary::Both
    }

    #[allow(unused)]
    fn is_boundary(&self) -> bool {
        *self != WordBoundary::Interior
    }
}

/// A cursor providing utility function to navigate the rope
/// by word boundaries.
/// Boundaries can be the start of a word, its end, punctuation etc.
// pub struct WordCursor<'a> {
//     pub(crate) inner: Cursor<'a, RopeInfo>,
// }

pub struct ModalWordCursor<'a> {
    pub(crate) inner: Cursor<'a, RopeInfo>,
}

pub trait WordCursor {
    fn prev_boundary(&mut self) -> Option<usize>;

    fn prev_deletion_boundary(&mut self) -> Option<usize>;

    fn next_boundary(&mut self) -> Option<usize>;

    fn end_boundary(&mut self) -> Option<usize>;

    fn prev_code_boundary(&mut self) -> usize;

    fn next_code_boundary(&mut self) -> usize;

    fn select_word(&mut self) -> (usize, usize);
}

impl<'a> ModalWordCursor<'a> {
    pub fn new(text: &'a Rope, pos: usize) -> ModalWordCursor<'a> {
        let inner = Cursor::new(text, pos);
        ModalWordCursor { inner }
    }

    /// Get the position of the next non blank character in the rope
    ///
    /// **Example:**
    ///
    /// ```rust
    /// # use lapce_core::word::{ WordCursor, ModalWordCursor };
    /// # use xi_rope::Rope;
    /// let rope = Rope::from("    world");
    /// let mut cursor = ModalWordCursor::new(&rope, 0);
    /// let char_position = cursor.next_non_blank_char();
    /// assert_eq!(char_position, 4);
    ///```
    pub fn next_non_blank_char(&mut self) -> usize {
        let mut candidate = self.inner.pos();
        while let Some(next) = self.inner.next_codepoint() {
            let prop = get_char_property(next);
            if prop != CharClassification::Space {
                break;
            }
            candidate = self.inner.pos();
        }
        self.inner.set(candidate);
        candidate
    }
}

impl<'a> WordCursor for ModalWordCursor<'a> {
    /// Get the next start boundary of a word, and set the cursor position to the boundary found.
    /// **Example:**
    ///
    /// ```rust
    /// # use lapce_core::word::{ ModalWordCursor, WordCursor };
    /// # use xi_rope::Rope;
    /// let rope = Rope::from("Hello world");
    /// let mut cursor = ModalWordCursor::new(&rope, 0);
    /// let boundary = cursor.next_boundary();
    /// assert_eq!(boundary, Some(6));
    ///```
    fn next_boundary(&mut self) -> Option<usize> {
        if let Some(ch) = self.inner.next_codepoint() {
            let mut prop = get_char_property(ch);
            let mut candidate = self.inner.pos();
            while let Some(next) = self.inner.next_codepoint() {
                let prop_next = get_char_property(next);
                if classify_boundary(prop, prop_next).is_start() {
                    break;
                }
                prop = prop_next;
                candidate = self.inner.pos();
            }
            self.inner.set(candidate);
            return Some(candidate);
        }
        None
    }

    /// Get the next end boundary, and set the cursor position to the boundary found.
    /// **Example:**
    ///
    /// ```rust
    /// # use lapce_core::word::{ WordCursor, ModalWordCursor };
    /// # use xi_rope::Rope;
    /// let rope = Rope::from("Hello world");
    /// let mut cursor = ModalWordCursor::new(&rope, 3);
    /// let end_boundary = cursor.end_boundary();
    /// assert_eq!(end_boundary, Some(5));
    ///```
    fn end_boundary(&mut self) -> Option<usize> {
        self.inner.next_codepoint();
        if let Some(ch) = self.inner.next_codepoint() {
            let mut prop = get_char_property(ch);
            let mut candidate = self.inner.pos();
            while let Some(next) = self.inner.next_codepoint() {
                let prop_next = get_char_property(next);
                if classify_boundary(prop, prop_next).is_end() {
                    break;
                }
                prop = prop_next;
                candidate = self.inner.pos();
            }
            self.inner.set(candidate);
            return Some(candidate);
        }
        None
    }

    /// Get the first matching [`CharClassification::Other`] backward and set the cursor position to this location .
    /// **Example:**
    ///
    /// ```rust
    /// # use lapce_core::word::{ WordCursor, ModalWordCursor };
    /// # use xi_rope::Rope;
    /// let text = "violet, are\n blue";
    /// let rope = Rope::from(text);
    /// let mut cursor = ModalWordCursor::new(&rope, 11);
    /// let position = cursor.prev_code_boundary();
    /// assert_eq!(&text[position..], "are\n blue");
    ///```
    fn prev_code_boundary(&mut self) -> usize {
        let mut candidate = self.inner.pos();
        while let Some(prev) = self.inner.prev_codepoint() {
            let prop_prev = get_char_property(prev);
            if prop_prev != CharClassification::Other {
                break;
            }
            candidate = self.inner.pos();
        }
        candidate
    }

    /// Get the first matching [`CharClassification::Other`] forward and set the cursor position to this location .
    /// **Example:**
    ///
    /// ```rust
    /// # use lapce_core::word::{ WordCursor, ModalWordCursor };
    /// # use xi_rope::Rope;
    /// let text = "violet, are\n blue";
    /// let rope = Rope::from(text);
    /// let mut cursor = ModalWordCursor::new(&rope, 11);
    /// let position = cursor.next_code_boundary();
    /// assert_eq!(&text[position..], "\n blue");
    ///```
    fn next_code_boundary(&mut self) -> usize {
        let mut candidate = self.inner.pos();
        while let Some(prev) = self.inner.next_codepoint() {
            let prop_prev = get_char_property(prev);
            if prop_prev != CharClassification::Other {
                break;
            }
            candidate = self.inner.pos();
        }
        candidate
    }

    /// Get the previous start boundary of a word, and set the cursor position to the boundary found.
    /// **Example:**
    ///
    /// ```rust
    /// # use lapce_core::word::{ WordCursor, ModalWordCursor };
    /// # use xi_rope::Rope;
    /// let rope = Rope::from("Hello world");
    /// let mut cursor = ModalWordCursor::new(&rope, 4);
    /// let boundary = cursor.prev_boundary();
    /// assert_eq!(boundary, Some(0));
    ///```
    fn prev_boundary(&mut self) -> Option<usize> {
        if let Some(ch) = self.inner.prev_codepoint() {
            let mut prop = get_char_property(ch);
            let mut candidate = self.inner.pos();
            while let Some(prev) = self.inner.prev_codepoint() {
                let prop_prev = get_char_property(prev);
                if classify_boundary(prop_prev, prop).is_start() {
                    break;
                }
                prop = prop_prev;
                candidate = self.inner.pos();
            }
            self.inner.set(candidate);
            return Some(candidate);
        }
        None
    }

    /// Computes where the cursor position should be after backward deletion.
    ///
    /// **Example:**
    ///
    /// ```rust
    /// # use lapce_core::word::{ WordCursor, ModalWordCursor };
    /// # use xi_rope::Rope;
    /// let text = "violet are blue";
    /// let rope = Rope::from(text);
    /// let mut cursor = ModalWordCursor::new(&rope, 9);
    /// let position = cursor.prev_deletion_boundary();
    /// let position = position;
    ///
    /// assert_eq!(position, Some(7));
    /// assert_eq!(&text[..position.unwrap()], "violet ");
    ///```
    fn prev_deletion_boundary(&mut self) -> Option<usize> {
        if let Some(ch) = self.inner.prev_codepoint() {
            let mut prop = get_char_property(ch);
            let mut candidate = self.inner.pos();

            // Flag, determines if the word should be deleted or not
            // If not, erase only whitespace characters.
            let mut keep_word = false;
            while let Some(prev) = self.inner.prev_codepoint() {
                let prop_prev = get_char_property(prev);

                // Stop if line beginning reached, without any non-whitespace characters
                if prop_prev == CharClassification::Lf
                    && prop == CharClassification::Space
                {
                    break;
                }

                // More than a single whitespace: keep word, remove only whitespaces
                if prop == CharClassification::Space
                    && prop_prev == CharClassification::Space
                {
                    keep_word = true;
                }

                // Line break found: keep words, delete line break & trailing whitespaces
                if prop == CharClassification::Lf || prop == CharClassification::Cr {
                    keep_word = true;
                }

                // Skip word deletion if above conditions were met
                if keep_word
                    && (prop_prev == CharClassification::Punctuation
                        || prop_prev == CharClassification::Other)
                {
                    break;
                }

                // Default deletion
                if classify_boundary(prop_prev, prop).is_start() {
                    break;
                }
                prop = prop_prev;
                candidate = self.inner.pos();
            }
            self.inner.set(candidate);
            return Some(candidate);
        }
        None
    }

    /// Return the previous and end boundaries of the word under cursor.
    ///
    /// **Example**:
    ///
    ///```rust
    /// # use lapce_core::word::{ WordCursor, ModalWordCursor };
    /// # use xi_rope::Rope;
    /// let text = "violet are blue";
    /// let rope = Rope::from(text);
    /// let mut cursor = ModalWordCursor::new(&rope, 9);
    /// let (start, end) = cursor.select_word();
    /// assert_eq!(&text[start..end], "are");
    ///```
    fn select_word(&mut self) -> (usize, usize) {
        let initial = self.inner.pos();
        let end = self.next_code_boundary();
        self.inner.set(initial);
        let start = self.prev_code_boundary();
        (start, end)
    }
}

/// Return the [`CharClassification`] of the input character
pub fn get_char_property(codepoint: char) -> CharClassification {
    if codepoint <= ' ' {
        if codepoint == '\r' {
            return CharClassification::Cr;
        }
        if codepoint == '\n' {
            return CharClassification::Lf;
        }
        return CharClassification::Space;
    } else if codepoint <= '\u{3f}' {
        if (0xfc00fffe00000000u64 >> (codepoint as u32)) & 1 != 0 {
            return CharClassification::Punctuation;
        }
    } else if codepoint <= '\u{7f}' {
        // Hardcoded: @[\]^`{|}~
        if (0x7800000178000001u64 >> ((codepoint as u32) & 0x3f)) & 1 != 0 {
            return CharClassification::Punctuation;
        }
    }
    CharClassification::Other
}

fn classify_boundary(
    prev: CharClassification,
    next: CharClassification,
) -> WordBoundary {
    use self::CharClassification::*;
    use self::WordBoundary::*;
    match (prev, next) {
        (Lf, Lf) => Start,
        (Lf, Space) => Interior,
        (Cr, Lf) => Interior,
        (Space, Lf) => Interior,
        (Space, Cr) => Interior,
        (Space, Space) => Interior,
        (_, Space) => End,
        (Space, _) => Start,
        (Lf, _) => Start,
        (_, Cr) => End,
        (_, Lf) => End,
        (Punctuation, Other) => Both,
        (Other, Punctuation) => Both,
        _ => Interior,
    }
}

#[cfg(test)]
mod test {
    use super::ModalWordCursor;
    use super::WordCursor;
    use xi_rope::Rope;

    #[test]
    fn prev_boundary_should_be_none_at_position_zero() {
        let rope = Rope::from("Hello world");
        let mut cursor = ModalWordCursor::new(&rope, 0);
        let boudary = cursor.prev_boundary();
        assert!(boudary.is_none())
    }

    #[test]
    fn prev_boundary_should_be_zero_when_cursor_on_first_word() {
        let rope = Rope::from("Hello world");
        let mut cursor = ModalWordCursor::new(&rope, 4);
        let boundary = cursor.prev_boundary();
        assert_eq!(boundary, Some(0));
    }

    #[test]
    fn prev_boundary_should_be_at_word_start() {
        let rope = Rope::from("Hello world");
        let mut cursor = ModalWordCursor::new(&rope, 9);
        let boundary = cursor.prev_boundary();
        assert_eq!(boundary, Some(6));
    }

    #[test]
    fn should_get_next_word_boundary() {
        let rope = Rope::from("Hello world");
        let mut cursor = ModalWordCursor::new(&rope, 0);
        let boundary = cursor.next_boundary();
        assert_eq!(boundary, Some(6));
    }

    #[test]
    fn next_word_boundary_should_be_none_at_last_position() {
        let rope = Rope::from("Hello world");
        let mut cursor = ModalWordCursor::new(&rope, 11);
        let boundary = cursor.next_boundary();
        assert_eq!(boundary, None);
    }

    #[test]
    fn should_get_previous_code_boundary() {
        let text = "violet, are\n blue";
        let rope = Rope::from(text);
        let mut cursor = ModalWordCursor::new(&rope, 11);
        let position = cursor.prev_code_boundary();
        assert_eq!(&text[position..], "are\n blue");
    }

    #[test]
    fn should_get_next_code_boundary() {
        let text = "violet, are\n blue";
        let rope = Rope::from(text);
        let mut cursor = ModalWordCursor::new(&rope, 11);
        let position = cursor.next_code_boundary();
        assert_eq!(&text[position..], "\n blue");
    }

    #[test]
    fn get_next_non_blank_char_should_skip_whitespace() {
        let rope = Rope::from("Hello world");
        let mut cursor = ModalWordCursor::new(&rope, 5);
        let char_position = cursor.next_non_blank_char();
        assert_eq!(char_position, 6);
    }

    #[test]
    fn get_next_non_blank_char_should_return_current_position_on_non_blank_char() {
        let rope = Rope::from("Hello world");
        let mut cursor = ModalWordCursor::new(&rope, 3);
        let char_position = cursor.next_non_blank_char();
        assert_eq!(char_position, 3);
    }

    #[test]
    fn should_get_end_boundary() {
        let rope = Rope::from("Hello world");
        let mut cursor = ModalWordCursor::new(&rope, 3);
        let end_boundary = cursor.end_boundary();
        assert_eq!(end_boundary, Some(5));
    }

    #[test]
    fn select_word_should_return_word_boundaries() {
        let text = "violet are blue";
        let rope = Rope::from(text);
        let mut cursor = ModalWordCursor::new(&rope, 9);
        let (start, end) = cursor.select_word();
        assert_eq!(&text[start..end], "are");
    }

    #[test]
    fn should_get_deletion_boundary_backward() {
        let text = "violet are blue";
        let rope = Rope::from(text);
        let mut cursor = ModalWordCursor::new(&rope, 9);
        let position = cursor.prev_deletion_boundary();
        let position = position;

        assert_eq!(position, Some(7));
        assert_eq!(&text[..position.unwrap()], "violet ");
    }
}
