use unicode_segmentation::*;

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum TermChar<'a> {
    CSI(Csi<'a>),
    Invisible(&'a str),
    Visible(&'a str)
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct TermString<'a> {
    characters: Vec<TermChar<'a>>,
    visible_chars_count: usize
}

#[derive(PartialEq, Eq, Debug, Clone)]
enum Either<L, R> {
    Left(L),
    Right(R)
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Csi<'a> {
    SGR(Vec<&'a str>),
    Other(Vec<&'a str>)
}

impl<'a> Csi<'a> {
    fn from_valid_graphemes(v: Vec<&'a str>) -> Csi<'a> {
        if v.last().unwrap() == &"m" {
            Csi::SGR(v)
        } else {
            Csi::Other(v)
        }
    }
}

fn try_take_csi<'a>(graphemes: &mut Graphemes<'a>) -> Either<Vec<&'a str>, Csi<'a>> {
    let mut buffer = vec!["\x1b", "["];
    let mut have_intermedia_bytes = false;
    while let Some(point) = graphemes.next() {
        buffer.push(point);
        if point.len() != 1 {
            break;
        }

        match point.as_bytes()[0] {
            // Final byte, End of CSI Sequence
            0x40..=0x7e => {
                return Either::Right(Csi::from_valid_graphemes(buffer))
            },
            // Intermedia bytes, can be any number
            0x20..=0x2f => {
                have_intermedia_bytes = true
            },
            // Parameter bytes, should not show up after intermedia bytes
            0x30..=0x3f => {
                if have_intermedia_bytes {
                    break
                }
            },
            _ => break
        }
    }
    Either::Left(buffer)
}

fn consume_grapheme_to<'a>(c: &'a str, chars: &mut Vec<TermChar<'a>>) {
    if c.len() == 1 {
        match c.as_bytes()[0] {
            0x09 | 0x0a | 0x0d |0x20..=0x7e => chars.push(TermChar::Visible(c)),
            _ => chars.push(TermChar::Invisible(c)),
        }
    } else {
        chars.push(TermChar::Visible(c))
    }
}

impl<'a> TermString<'a> {

    pub fn new(s: &'a str, strict: bool) -> Option<TermString<'a>> {

        let mut characters = vec![];

        let mut graphemes = s.graphemes(true);

        loop {
            match graphemes.next() {
                None => break,
                Some("\x1b") => {
                    if let Some("[") = graphemes.next() {
                        match try_take_csi(&mut graphemes) {
                            Either::Right(csi)   => characters.push(TermChar::CSI(csi)),
                            Either::Left(failed) => {
                                if strict {
                                    return None;
                                }
                                consume_grapheme_to("\x1b", &mut characters);
                                consume_grapheme_to("[", &mut characters);
                                for c in failed.iter() {
                                    consume_grapheme_to(c, &mut characters)
                                }
                            }
                        }
                    }
                },
                Some(g) => consume_grapheme_to(g, &mut characters)
            }
        }

        let mut visible_chars_count = 0;

        let mut char_iter = characters.iter();
        while let Some(c) = char_iter.next() {
            if let TermChar::Visible(_) = c {
                visible_chars_count += 1;
            }
        }

        Some(TermString { characters, visible_chars_count })
    }

    pub fn visible_chars_count(self) -> usize {
        self.visible_chars_count
    }

    pub fn pad_left_and_truncate(&self, size: usize, padchar: char) -> String {
        let mut ret = String::new();
        let chars_to_pad = if self.visible_chars_count >= size {
                0
            } else {
                size - self.visible_chars_count
            };
        ret.push_str(&padchar.to_string().repeat(chars_to_pad));

        // If the string we have is >= then requested size, we won't be padding
        // at all, hence we can simply append the truncated string to the end.
        // If the string we have is < requested size, truncate basically just
        // return the entire string
        ret.push_str(&self.truncated(size));
        ret
    }

    pub fn truncated(&self, size: usize) -> String {
        let mut ret = String::new();
        let mut iterator = self.characters.iter();

        let mut count = 0;

        while let Some(next) = iterator.next() {
            match next {
                TermChar::Visible(c) => {
                    ret.push_str(c);
                    count += 1;
                    // We only check the condition here because if there are
                    // trailing color formatting seqs we wanna include it
                    if count == size {
                        break
                    }
                },
                TermChar::CSI(Csi::SGR(seq)) => ret.push_str(&seq.concat()),
                _ => continue
            }
        }

        ret
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! colored_snake {
        () => {
            vec![
                TermChar::Invisible("\x1f"),
                TermChar::CSI(Csi::SGR(vec!["\x1b","[","9","3","m"])),
                TermChar::Visible("s"), TermChar::Visible("n"), TermChar::Visible("a"), 
                TermChar::Visible("k"), TermChar::Visible("e"),
                TermChar::CSI(Csi::SGR(vec!["\x1b","[","0","m"]))
            ]
        }
    }

    const INPUT: &'static str = "\x1f\x1b[93msnake\x1b[0m";

    #[test]
    fn test_internal_structure() {
        let string = TermString::new(INPUT, false).unwrap();
        assert_eq!(colored_snake!(), string.characters);
        assert_eq!(string.visible_chars_count, 5);
    }

    #[test]
    fn truncate_lt_size() {
        let string = TermString::new(INPUT, false).unwrap();
        assert_eq!(string.truncated(4), "\x1b[93msnak".to_string());
    }

    #[test]
    fn truncating_gt_size() {
        let string = TermString::new(INPUT, false).unwrap();
        assert_eq!(string.truncated(100).as_str(),  "\x1b[93msnake\x1b[0m");
    }
}
