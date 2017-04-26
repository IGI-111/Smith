use std::string::String;
use xi_rope::Rope;
use std::fs::File;
use std::cmp;
use std::io::{Read, Write, Result, Error, ErrorKind};
use std::path::Path;
use super::{Movement, Editable, Named, Saveable, Lines};

pub struct Text {
    pub pos: usize,
    pub text: Rope,
    pub name: String,
}

impl Text {
    pub fn empty() -> Text {
        Text {
            pos: 0,
            text: Rope::from("\n"),
            name: String::new(),
        }
    }
    pub fn open_file(filename: String) -> Result<Text> {
        if Path::new(&filename).exists() {
            let mut file = File::open(&filename)?;

            let mut buf = String::new();
            file.read_to_string(&mut buf)?;

            match buf.chars().last() {
                Some('\n') => {}
                _ => buf.push('\n'),
            }
            let text = Rope::from(buf);

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

    fn char_at(&self, pos: usize) -> char {
        self
            .slice(pos, pos + 1)
            .chars()
            .next()
            .unwrap()
    }

    fn delete_at(&mut self, pos: usize) {
        self.delete_range(pos, pos + 1);
    }

    fn insert_at(&mut self, text: &str, pos: usize) {
        self.text.edit_str(pos, pos, text);
    }
}

impl Saveable for Text {
    fn save(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(Error::new(ErrorKind::InvalidInput, "Can't write file with no name"));
        }
        let mut file = File::create(&self.name)?;
        for c in self.text.iter_chunks() {
            write!(file, "{}", c)?;
        }
        file.sync_all()?;
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
                    let prev_line = self.text.offset_of_line(self.line() - 1);
                    let prev_line_size = self.text.lines().nth(self.line() - 1).unwrap().len();
                    self.pos = prev_line + cmp::min(self.col(), prev_line_size - 1);
                }
            }
            Movement::Down => {
                if self.line() < self.line_count() - 1 {
                    let next_line = self.text.offset_of_line(self.line() + 1);
                    let next_line_size = self.text.lines().nth(self.line() + 1).unwrap().len();
                    self.pos = next_line + cmp::min(self.col(), next_line_size - 1);
                }
            }
            Movement::PageUp(up) => {
                let target_line = if self.line() < up {
                    0
                } else {
                    self.line() - up
                };
                self.pos = self.text.offset_of_line(target_line);
            }
            Movement::PageDown(down) => {
                let target_line = if self.line_count() - self.line() < down {
                    self.line_count() - 1
                } else {
                    self.line() + down
                };
                self.pos = self.text.offset_of_line(target_line);
            }
            Movement::Left => {
                if self.pos > 0 {
                    self.pos -= 1;
                }
            }
            Movement::Right => {
                if self.pos < self.text.len() - 1 {
                    self.pos += 1;
                }
            }
            Movement::LineStart => {
                let curr_line = self.text.offset_of_line(self.line());

                self.pos = curr_line;
            }
            Movement::LineEnd => {
                let curr_line = self.text.offset_of_line(self.line());
                let curr_line_size = self.text.lines().nth(self.line()).unwrap().len();
                self.pos = curr_line + curr_line_size - 1;
            }
        }
    }

    fn insert(&mut self, c: char) {
        let pos = self.pos;
        self.insert_at(&format!("{}", c), pos);
        self.pos += 1;
    }
    fn insert_forward(&mut self, c: char) {
        let pos = self.pos;
        self.insert_at(&format!("{}", c), pos);
    }

    fn delete(&mut self) -> Option<char> {
        if self.pos == 0 {
            None
        } else {
            self.pos -= 1;
            let ch = self.char_at(self.pos);
            let pos = self.pos;
            self.delete_at(pos);
            self.text.edit_str(self.pos, self.pos + 1, "");
            Some(ch)
        }
    }

    fn delete_forward(&mut self) -> Option<char> {
        if self.pos < self.len() - 1 {
            let ch = self.char_at(self.pos);
            let pos = self.pos;
            self.delete_at(pos);
            Some(ch)
        } else {
            None
        }
    }

    fn move_to(&mut self, pos: usize) {
        assert!(pos < self.text.len());
        self.pos = pos;
    }

    fn move_at(&mut self, line: usize, col: usize) {
        let line = cmp::min(line, self.line_count() - 1);
        let col = cmp::min(col, self.text.lines().nth(line).unwrap().len() - 1);
        self.pos = self.text.offset_of_line(line) + col;
    }

    fn pos(&self) -> usize {
        self.pos
    }

    fn line(&self) -> usize {
        self.text.line_of_offset(self.pos)
    }

    fn col(&self) -> usize {
        self.pos - self.text.offset_of_line(self.line())
    }

    fn line_count(&self) -> usize {
        self.text.line_of_offset(self.text.len() - 1) + 1
    }

    fn len(&self) -> usize {
        self.text.len()
    }

    fn lines(&self) -> Lines {
        self.text.lines()
    }

    fn slice(&self, start: usize, end: usize) -> String {
        let text = self.text.clone();
        String::from(text.slice(start, end))
    }

    fn delete_range(&mut self, start: usize, end: usize) {
        self.text.edit_str(start, end, "");
    }

    fn offset_of_line(&self, line: usize) -> usize {
        self.text.offset_of_line(line)
    }
}
