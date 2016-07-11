use std::string::String;

pub struct Position {
    pub line: usize,
    pub column: usize,
}

impl Position {
    pub fn new(line: usize, column: usize) -> Position {
        Position {line:line, column:column}
    }
}


pub struct Text {
    pos: Position,
    lines: Vec<String>,
}

impl Text {
    pub fn new() -> Text {
        Text { pos: Position::new(0,0), lines:vec![
            String::from("Lorem ipsum dolor sit amet, consectetur adipisicing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum. "),
            String::from("Lorem"),
            String::from("Ipsum")] }
    }
    pub fn get_pos(&self) -> &Position {
        &self.pos
    }
    pub fn get_lines(&self) -> &Vec<String> {
        &self.lines
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
                if self.pos.line < self.lines.len()-1 {
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

    fn check_column(&mut self){
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

    pub fn delete(&mut self){
        if self.pos.column == 0 && self.pos.line > 0 {
            let previous_line_end = self.lines[self.pos.line-1].len();
            {
                let line_content = self.lines[self.pos.line].clone();
                let previous_line = &mut self.lines[self.pos.line-1];
                previous_line.push_str(&line_content);
            }
            self.lines.remove(self.pos.line);

            self.pos.line -= 1;
            self.pos.column = previous_line_end;
        } else {
            self.lines[self.pos.line].remove(self.pos.column-1);
            self.pos.column -= 1;
        }
    }

    pub fn new_line(&mut self){
        if self.pos.column == self.lines[self.pos.line].len() {
            self.lines.insert(self.pos.line+1, String::new());
            self.pos.line += 1;
            self.check_column();
        } else {
            let old_line;
            let new_line;
            {
                let line_split =
                    self.lines[self.pos.line].split_at(self.pos.column).clone();
                old_line = String::from(line_split.0);
                new_line = String::from(line_split.1);
            }
            self.lines.insert(self.pos.line+1, String::from(new_line));
            self.lines[self.pos.line] = String::from(old_line);

            self.pos.line += 1;
            self.pos.column = 0;
        }

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
