use crate::syntax::util::{matching_char, matching_pair_direction};
use xi_rope::{Cursor, Rope, RopeInfo};

pub struct BracketCursor<'a> {
    pub(crate) inner: Cursor<'a, RopeInfo>,
}

impl<'a> BracketCursor<'a> {
    pub fn new(text: &'a Rope, pos: usize) -> BracketCursor<'a> {
        let inner = Cursor::new(text, pos);
        BracketCursor { inner }
    }

    /// Looks for a matching pair character, either forward for opening chars (ex: `(`) or
    /// backward for closing char (ex: `}`), and return the matched character position if found.
    /// Will return `None` if the character under cursor is not matchable (see [`crate::syntax::util::matching_char`]).
    ///
    /// **Example:**
    ///
    /// ```rust
    /// # use lapce_core::bracket::BracketCursor;
    /// # use xi_rope::Rope;
    /// let text = "{ }";
    /// let rope = Rope::from(text);
    /// let mut cursor = BracketCursor::new(&rope, 2);
    /// let position = cursor.match_pairs();
    /// assert_eq!(position, Some(0));
    ///```
    pub fn match_pairs(&mut self) -> Option<usize> {
        let c = self.inner.peek_next_codepoint()?;
        let other = matching_char(c)?;
        let left = matching_pair_direction(other)?;
        if left {
            self.previous_unmatched(other)
        } else {
            self.inner.next_codepoint();
            let offset = self.next_unmatched(other)?;
            Some(offset - 1)
        }
    }

    /// Take a matchable character and look cforward for the first unmatched one
    /// ignoring the encountered matched pairs.
    ///
    /// **Example**:
    /// ```rust
    /// # use xi_rope::Rope;
    /// # use lapce_core::bracket::BracketCursor;
    /// let rope = Rope::from("outer {inner}} world");
    /// let mut cursor = BracketCursor::new(&rope, 0);
    /// let position = cursor.next_unmatched('}');
    /// assert_eq!(position, Some(14));
    ///  ```
    pub fn next_unmatched(&mut self, c: char) -> Option<usize> {
        let other = matching_char(c)?;
        let mut n = 0;
        while let Some(current) = self.inner.next_codepoint() {
            if current == c && n == 0 {
                return Some(self.inner.pos());
            }
            if current == other {
                n += 1;
            } else if current == c {
                n -= 1;
            }
        }
        None
    }

    /// Take a matchable character and look backward for the first unmatched one
    /// ignoring the encountered matched pairs.
    ///
    /// **Example**:
    ///
    /// ```rust
    /// # use xi_rope::Rope;
    /// # use lapce_core::bracket::BracketCursor;
    /// let rope = Rope::from("outer {{inner} world");
    /// let mut cursor = BracketCursor::new(&rope, 15);
    /// let position = cursor.previous_unmatched('{');
    /// assert_eq!(position, Some(6));
    ///  ```
    pub fn previous_unmatched(&mut self, c: char) -> Option<usize> {
        let other = matching_char(c)?;
        let mut n = 0;
        while let Some(current) = self.inner.prev_codepoint() {
            if current == c && n == 0 {
                return Some(self.inner.pos());
            }
            if current == other {
                n += 1;
            } else if current == c {
                n -= 1;
            }
        }
        None
    }
}

#[cfg(test)]
mod test {
    use crate::bracket::BracketCursor;
    use xi_rope::Rope;

    #[test]
    fn should_get_next_unmatched_char() {
        let rope = Rope::from("hello { world");
        let mut cursor = BracketCursor::new(&rope, 0);
        let position = cursor.next_unmatched('{');
        assert_eq!(position, Some(7));
    }

    #[test]
    fn should_get_next_unmatched_char_witch_matched_chars() {
        let rope = Rope::from("hello {} world }");
        let mut cursor = BracketCursor::new(&rope, 0);
        let position = cursor.next_unmatched('}');
        assert_eq!(position, Some(16));
    }

    #[test]
    fn should_get_previous_unmatched_char() {
        let rope = Rope::from("hello { world");
        let mut cursor = BracketCursor::new(&rope, 12);
        let position = cursor.previous_unmatched('{');
        assert_eq!(position, Some(6));
    }

    #[test]
    fn should_get_previous_unmatched_char_with_inner_matched_chars() {
        let rope = Rope::from("{hello {} world");
        let mut cursor = BracketCursor::new(&rope, 10);
        let position = cursor.previous_unmatched('{');
        assert_eq!(position, Some(0));
    }

    #[test]
    fn should_match_pair_forward() {
        let text = "{ }";
        let rope = Rope::from(text);
        let mut cursor = BracketCursor::new(&rope, 0);
        let position = cursor.match_pairs();
        assert_eq!(position, Some(2));
    }

    #[test]
    fn should_match_pair_backward() {
        let text = "{ }";
        let rope = Rope::from(text);
        let mut cursor = BracketCursor::new(&rope, 2);
        let position = cursor.match_pairs();
        assert_eq!(position, Some(0));
    }

    #[test]
    fn match_pair_should_be_none() {
        let text = "{ }";
        let rope = Rope::from(text);
        let mut cursor = BracketCursor::new(&rope, 1);
        let position = cursor.match_pairs();
        assert_eq!(position, None);
    }
}
