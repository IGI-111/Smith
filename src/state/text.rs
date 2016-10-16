use std::string::String;
use ropey::Rope;
use std::fs::File;
use std::cmp;
use std::io::{Read, Write, Result, Error, ErrorKind};
use std::path::Path;
use super::{Movement, Editable, Named, Saveable, CharIter};

#[derive(Debug)]
pub struct Text {
    pub pos: usize,
    pub text: Rope,
    pub name: String,
}

impl Text {
    pub fn empty() -> Text {
        Text {
            pos: 0,
            text: Rope::from_str("\n"),
            name: String::new(),
        }
    }
    pub fn open_file(filename: String) -> Result<Text> {
        if Path::new(&filename).exists() {
            let mut file = try!(File::open(&filename));

            let mut buf = String::new();
            try!(file.read_to_string(&mut buf));
            if buf.is_empty() {
                buf = "\n".to_owned();
            }
            let text = Rope::from_string(buf);

            Ok(Text {
                pos: 0,
                text: text,
                name: filename,
            })
        } else {
            let mut text = Text::empty();
            text.set_name(filename);
            Ok(text)
        }
    }
}

impl Saveable for Text {
    fn save(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(Error::new(ErrorKind::InvalidInput, "Can't write file with no name"));
        }
        let mut file = try!(File::create(&self.name));
        for c in self.text.char_iter() {
            try!(write!(file, "{}", c));
        }
        try!(file.sync_all());
        Ok(())
    }
}

impl Named for Text {
    fn name(&self) -> &String {
        &self.name
    }
    fn set_name(&mut self, name: String) {
        self.name = name;
    }
}

impl Editable for Text {
    fn step(&mut self, mov: Movement) {
        match mov {
            Movement::Up => {
                if self.line() > 0 {
                    let prev_line = self.text.line_index_to_char_index(self.line() - 1);
                    let prev_line_size =
                        self.text.line_iter().nth(self.line() - 1).unwrap().char_count();
                    self.pos = prev_line + cmp::min(self.col(), prev_line_size - 1);
                }
            }
            Movement::Down => {
                if self.line() < self.line_count() - 1 {
                    let next_line = self.text.line_index_to_char_index(self.line() + 1);
                    let next_line_size =
                        self.text.line_iter().nth(self.line() + 1).unwrap().char_count();
                    self.pos = next_line + cmp::min(self.col(), next_line_size - 1);
                }
            }
            Movement::PageUp(up) => {
                let target_line = if self.line() < up {
                    0
                } else {
                    self.line() - up
                };
                self.pos = self.text.line_index_to_char_index(target_line);
            }
            Movement::PageDown(down) => {
                let target_line = if self.line_count() - self.line() < down {
                    self.line_count() - 1
                } else {
                    self.line() + down
                };
                self.pos = self.text.line_index_to_char_index(target_line);
            }
            Movement::Left => {
                if self.pos > 0 {
                    self.pos -= 1;
                }
            }
            Movement::Right => {
                if self.pos < self.text.char_count() - 1 {
                    self.pos += 1;
                }
            }
            Movement::LineStart => {
                let curr_line = self.text.line_index_to_char_index(self.line());

                self.pos = curr_line;
            }
            Movement::LineEnd => {
                let curr_line = self.text.line_index_to_char_index(self.line());
                let curr_line_size = self.text.line_iter().nth(self.line()).unwrap().char_count();
                self.pos = curr_line + curr_line_size - 1;
            }
        }
    }

    fn insert(&mut self, c: char) {
        self.text.insert_text_at_char_index(&format!("{}", c), self.pos);
        self.pos += 1;
    }
    fn insert_forward(&mut self, c: char) {
        self.text.insert_text_at_char_index(&format!("{}", c), self.pos);
    }

    fn delete(&mut self) -> Option<char> {
        if self.pos == 0 {
            None
        } else {
            self.pos -= 1;
            let ch = self.text.char_at_index(self.pos);
            self.text.remove_text_between_char_indices(self.pos, self.pos + 1);
            Some(ch)
        }
    }

    fn delete_forward(&mut self) -> Option<char> {
        if self.pos < self.len()-1 {
            let ch = self.text.char_at_index(self.pos);
            self.text.remove_text_between_char_indices(self.pos, self.pos + 1);
            Some(ch)
        } else {
            None
        }
    }

    fn move_to(&mut self, pos: usize) {
        assert!(pos < self.text.char_count());
        self.pos = pos;
    }


    fn move_at(&mut self, line: usize, col: usize) {
        let line = cmp::min(line, self.line_count() - 1);
        let col = cmp::min(col,
                           self.text.line_iter().nth(line).unwrap().char_count() - 1);
        self.pos = self.text.line_index_to_char_index(line) + col;
    }

    fn pos(&self) -> usize {
        self.pos
    }

    fn line(&self) -> usize {
        self.text.char_index_to_line_index(self.pos)
    }

    fn col(&self) -> usize {
        self.pos - self.text.line_index_to_char_index(self.line())
    }

    fn line_count(&self) -> usize {
        self.text.line_ending_count()
    }

    fn len(&self) -> usize {
        self.text.char_count()
    }

    fn iter(&self) -> CharIter {
        self.text.char_iter()
    }

    fn iter_line(&self, line: usize) -> CharIter {
        self.text.line_iter_at_index(line).next().unwrap().char_iter()
    }

    fn line_index_to_char_index(&self, line: usize) -> usize {
        self.text.line_index_to_char_index(line)
    }
}
