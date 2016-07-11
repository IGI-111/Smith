use std::string::String;

pub struct Text {
    pos: (usize, usize),
    // line, column

    lines: Vec<String>,
}

impl Text {
    pub fn new() -> Text {
        Text { pos:(1, 3), lines:vec![
                     String::from("Lorem ipsum dolor sit amet, consectetur adipisicing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum. "),
                     String::from("Lorem"),
                     String::from("Ipsum")] }
    }
    pub fn get_pos(&self) -> (usize, usize) {
        self.pos
    }
    pub fn get_lines(&self) -> &Vec<String> {
        &self.lines
    }
    pub fn step(&mut self, mov: Movement) {
        match mov {
            Movement::Up => {
                if self.pos.0 > 0 {
                    self.pos.0 -= 1;
                    self.check_column();
                }
            }
            Movement::Down => {
                if self.pos.0 < self.lines.len()-1 {
                    self.pos.0 += 1;
                    self.check_column();
                }
            }
            Movement::Left => {
                if self.pos.1 > 0 {
                    self.pos.1 -= 1;
                }
            }
            Movement::Right => {
                if self.pos.1 < self.lines[self.pos.0].len() {
                    self.pos.1 += 1;
                }
            }
            Movement::LineStart => {
                self.pos.1 = 0;
            }
            Movement::LineEnd => {
                self.pos.1 = self.lines[self.pos.0].len();
            }
        }
    }

    fn check_column(&mut self){
        let line_length = self.lines[self.pos.0].len();
        if self.pos.1 > line_length {
            self.pos.1 = line_length;
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
