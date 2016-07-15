use std::string::String;
use std::fs::File;
use std::io::{Read, Write, Result, Error, ErrorKind};
use std::path::Path;
use super::{Movement, Position, Editable, Named, Saveable};

#[derive(Debug)]
pub struct Text {
    pub pos: Position,
    pub lines: Vec<String>,
    pub name: String,
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
}

impl Saveable for Text {
    fn save(&self) -> Result<()> {
        if self.name.len() <= 0 {
            return Err(Error::new(ErrorKind::InvalidInput, "Can't write file with no name"));
        }
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

impl Named for Text {
    fn name(&self) -> &String {
        &self.name
    }
}

impl Editable for Text {
    fn step(&mut self, mov: Movement) {
        println!("{:?}", self);
        match mov {
            Movement::Up => {
                if self.pos.line > 0 {
                    self.pos.line -= 1;
                    check_column(self);
                }
            }
            Movement::Down => {
                if self.pos.line < self.lines.len() - 1 {
                    self.pos.line += 1;
                    check_column(self);
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

    fn insert(&mut self, c: char) {
        if c == '\n' {
            new_line(self);
        } else {
            let ref mut line_string = self.lines[self.pos.line];
            let index = line_string.char_indices().nth(self.pos.column);
            match index {
                None => line_string.push(c),
                Some((i, _)) => line_string.insert(i, c),
            };
            self.pos.column += 1;
        }
    }

    fn delete(&mut self) -> Option<char> {
        if self.pos.column != 0 {
            let c = self.lines[self.pos.line].remove(self.pos.column - 1);
            self.pos.column -= 1;
            Some(c)
        } else if self.pos.line != 0 {
            join_line(self);
            Some('\n')
        } else {
            None
        }

    }

    fn move_to(&mut self, pos: Position) {
        // FIXME add checks
        self.pos = pos;
    }

    fn pos(&self) -> &Position {
        &self.pos
    }

    fn lines(&self) -> &Vec<String> {
        &self.lines
    }
}

fn join_line(text: &mut Text) {
    let previous_line_end = text.lines[text.pos.line - 1].len();
    {
        let line_content = text.lines[text.pos.line].clone();
        let ref mut previous_line = text.lines[text.pos.line - 1];
        previous_line.push_str(&line_content);
    }
    text.lines.remove(text.pos.line);

    text.pos.line -= 1;
    text.pos.column = previous_line_end;
}

fn new_line(text: &mut Text) {
    if text.pos.column == text.lines[text.pos.line].len() {
        text.lines.insert(text.pos.line + 1, String::new());
        text.pos.line += 1;
        check_column(text);
    } else {
        let old_line;
        let new_line;
        {
            let line_split = text.lines[text.pos.line].split_at(text.pos.column).clone();
            old_line = String::from(line_split.0);
            new_line = String::from(line_split.1);
        }
        text.lines.insert(text.pos.line + 1, String::from(new_line));
        text.lines[text.pos.line] = String::from(old_line);

        text.pos.line += 1;
        text.pos.column = 0;
    }

}

fn check_column(text: &mut Text) {
    let line_length = text.lines[text.pos.line].len();
    if text.pos.column > line_length {
        text.pos.column = line_length;
    }
}
