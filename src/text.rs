use std::string::String;
use std::fs::File;
use std::io::{Read, Write, Result};
use std::path::Path;

pub struct Position {
    pub line: usize,
    pub column: usize,
}

impl Position {
    pub fn new(line: usize, column: usize) -> Position {
        Position {
            line: line,
            column: column,
        }
    }
}


pub struct Text {
    pos: Position,
    lines: Vec<String>,
    name: String,
}

impl Text {
    pub fn empty() -> Text {
        Text {
            pos: Position::new(0, 0),
            lines: vec![String::new()],
            name: String::new(),
        }
    }
    pub fn open_file(filename: String) -> Result<Text> {
        if Path::new(&filename).exists() {
            let mut file = try!(File::open(&filename));

            let mut buf = String::new();
            try!(file.read_to_string(&mut buf));
            let lines: Vec<String> = buf.split_terminator("\n").map(|x| String::from(x)).collect();

            Ok(Text {
                pos: Position::new(0, 0),
                lines: lines,
                name: filename,
            })
        } else {
            Ok(Text {
                pos: Position::new(0, 0),
                lines: vec![String::new()],
                name: filename,
            })
        }
    }
    pub fn get_pos(&self) -> &Position {
        &self.pos
    }
    pub fn get_lines(&self) -> &Vec<String> {
        &self.lines
    }
    pub fn get_name(&self) -> &String {
        &self.name
    }
    pub fn step(&mut self, mov: Movement) {
        match mov {
            Movement::Up => {
                if self.pos.line > 0 {
                    self.pos.line -= 1;
                    self.check_column();
                }
            }
            Movement::Down => {
                if self.pos.line < self.lines.len() - 1 {
                    self.pos.line += 1;
                    self.check_column();
                }
            }
            Movement::Left => {
                if self.pos.column > 0 {
                    self.pos.column -= 1;
                }
            }
            Movement::Right => {
                if self.pos.column < self.lines[self.pos.line].len() {
                    self.pos.column += 1;
                }
            }
            Movement::LineStart => {
                self.pos.column = 0;
            }
            Movement::LineEnd => {
                self.pos.column = self.lines[self.pos.line].len();
            }
        }
    }

    fn check_column(&mut self) {
        let line_length = self.lines[self.pos.line].len();
        if self.pos.column > line_length {
            self.pos.column = line_length;
        }
    }

    pub fn insert(&mut self, c: char) {
        let line_string = &mut self.lines[self.pos.line];
        if self.pos.column == line_string.len() {
            line_string.push(c);
        } else {
            line_string.insert(self.pos.column, c);
        }
        self.pos.column += 1;
    }

    pub fn delete(&mut self) {
        if self.pos.column == 0 {
            if self.pos.line == 0 {
                return; // can't delete last line
            }
            let previous_line_end = self.lines[self.pos.line - 1].len();
            {
                let line_content = self.lines[self.pos.line].clone();
                let previous_line = &mut self.lines[self.pos.line - 1];
                previous_line.push_str(&line_content);
            }
            self.lines.remove(self.pos.line);

            self.pos.line -= 1;
            self.pos.column = previous_line_end;
        } else {
            self.lines[self.pos.line].remove(self.pos.column - 1);
            self.pos.column -= 1;
        }
    }

    pub fn new_line(&mut self) {
        if self.pos.column == self.lines[self.pos.line].len() {
            self.lines.insert(self.pos.line + 1, String::new());
            self.pos.line += 1;
            self.check_column();
        } else {
            let old_line;
            let new_line;
            {
                let line_split = self.lines[self.pos.line].split_at(self.pos.column).clone();
                old_line = String::from(line_split.0);
                new_line = String::from(line_split.1);
            }
            self.lines.insert(self.pos.line + 1, String::from(new_line));
            self.lines[self.pos.line] = String::from(old_line);

            self.pos.line += 1;
            self.pos.column = 0;
        }

    }
    pub fn save_file(&self) -> Result<()> {
        let mut file = try!(File::create(&self.name));
        for line in self.lines.iter() {
            let mut line_bytes = Vec::from(line.as_bytes());
            line_bytes.extend(b"\n");
            try!(file.write_all(line_bytes.as_slice()));
        }
        try!(file.sync_all());
        Ok(())
    }
}

pub enum Movement {
    Up,
    Down,
    Left,
    Right,
    LineStart,
    LineEnd,
}
