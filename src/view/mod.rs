mod screen;

use self::screen::Screen;
use data::{Editable, Modifiable, Named, Selectable, Undoable};
use std::{cmp, iter};
use syntect::easy::HighlightLines;
use syntect::highlighting::Theme;
use syntect::highlighting::{Color, FontStyle, Style};
use syntect::parsing::{SyntaxReference, SyntaxSet};
use termion::terminal_size;
pub struct View<'a> {
    message: Option<String>,
    is_prompt: bool,
    line_offset: usize,
    screen: Screen,
    theme: &'a Theme,
    syntax_set: &'a SyntaxSet,
    syntax_ref: &'a SyntaxReference,
}

const TAB_LENGTH: usize = 4;

impl<'a> View<'a> {
    pub fn new(
        theme: &'a Theme,
        syntax_ref: &'a SyntaxReference,
        syntax_set: &'a SyntaxSet,
    ) -> Self {
        let default_style = Style {
            foreground: theme.settings.foreground.unwrap_or(Color::WHITE),
            background: theme.settings.background.unwrap_or(Color::BLACK),
            font_style: FontStyle::empty(),
        };
        View {
            message: None,
            is_prompt: false,
            line_offset: 0,
            screen: Screen::with_default_style(default_style),
            theme,
            syntax_set,
            syntax_ref,
        }
    }

    pub fn message(&mut self, message: &str) {
        self.is_prompt = false;
        self.message = Some(String::from(message));
    }

    pub fn prompt(&mut self, prompt: &str, message: &str) {
        self.is_prompt = true;
        let msg = String::from(prompt) + message;
        self.message = Some(msg);
    }

    pub fn quiet(&mut self) {
        self.is_prompt = false;
        self.message = None;
    }

    pub fn center_view(&mut self, line: usize) {
        self.line_offset = line
            .checked_sub(self.lines_height() as usize / 2)
            .unwrap_or(0);
    }

    pub fn adjust_view(&mut self, line: usize) {
        if line < self.line_offset {
            self.line_offset = line;
        } else if line + 1 >= self.line_offset + self.lines_height() {
            self.line_offset = 1 + line - self.lines_height();
        }
    }

    pub fn scroll_view<T: Editable>(&mut self, offset: isize, content: &T) {
        self.line_offset = cmp::min(
            cmp::max((self.line_offset as isize) + offset, 0),
            (content.line_count() as isize) - 1,
        ) as usize;
    }

    pub fn render<T>(&mut self, content: &T)
    where
        T: Editable + Named + Selectable + Undoable + Modifiable,
    {
        self.screen.clear();
        self.paint_lines(content);
        self.paint_status(content);
        self.paint_message();
        self.paint_cursor(content);
        self.screen.present();
    }

    pub fn translate_coordinates<T>(&self, content: &T, x: u16, y: u16) -> (usize, usize)
    where
        T: Editable,
    {
        let line = cmp::min(
            (y as isize + self.line_offset as isize - 1) as usize,
            content.line_count() - 1,
        );
        let visual_col = (cmp::max(
            0,
            x as isize - self.line_number_width(content.line_count()) as isize - 2,
        )) as usize;
        // find out if we clicked through a tab
        let col = content
            .iter_line(line)
            .scan(0, |state, x| {
                *state += if x == '\t' { TAB_LENGTH } else { 1 };
                Some(*state)
            })
            .take_while(|&x| x <= visual_col)
            .count();
        (line, col)
    }

    fn paint_message(&self) {
        if let Some(ref message) = self.message {
            let y = self.lines_height() + 1;
            self.screen.draw(0, y, message);
        }
    }

    fn paint_cursor<T>(&mut self, content: &T)
    where
        T: Editable + Selectable,
    {
        // FIXME: don't print the cursor if off screen, though we should in the future for long
        // lines
        if (content.line()) < self.line_offset
            || content.line() >= self.line_offset + self.lines_height()
            || content.col() >= self.lines_width(content.line_count())
            || content.sel().is_some()
        {
            self.screen.hide_cursor();
            return;
        }

        // in the case of a prompt, the cursor should be drawn in the message line
        let (x, y) = if self.is_prompt {
            (
                self.message.clone().unwrap().chars().count(),
                self.lines_height() + 1,
            )
        } else {
            let (a, b) = self.cursor_pos(content);
            (a, b)
        };
        self.screen.move_cursor(x, y);
        self.screen.show_cursor();
    }

    fn paint_status<T>(&self, content: &T)
    where
        T: Editable + Named + Undoable + Modifiable,
    {
        let line = content.line();
        let column = content.col();
        let line_count = content.line_count();
        let advance = ((line + 1) as f64 / line_count as f64 * 100.0).floor();

        let (screen_width, _) = terminal_size().unwrap();
        let empty_line = (0..screen_width).map(|_| ' ').collect::<String>();
        let y = self.lines_height();

        let style = Style {
            background: self.theme.settings.background.unwrap_or(Color::BLACK),
            foreground: self.theme.settings.foreground.unwrap_or(Color::WHITE),
            font_style: FontStyle::empty(),
        };

        self.screen.draw_with_style(0, y, style, &empty_line);
        let mut filename = content.name().clone();
        if content.was_modified() {
            filename.push_str(" *");
        }
        self.screen.draw_with_style(0, y, style, &filename);

        let position_info = format!("{}% {}/{}: {}", advance, line + 1, line_count, column);
        let x = screen_width as usize - position_info.len();
        self.screen.draw_with_style(x, y, style, &position_info);
    }

    fn paint_lines<T>(&mut self, content: &T)
    where
        T: Editable + Selectable,
    {
        let line_offset = self.line_offset as usize;
        let lines_height = self.lines_height() as usize;
        let line_count = content.line_count();

        let line_start = self.line_number_width(line_count) as usize + 1;

        let mut highlighter = HighlightLines::new(self.syntax_ref, self.theme);

        for (i, line) in content.lines().enumerate() {
            let mut line_str = line
                .chars()
                .flat_map(|c| {
                    if c == '\t' {
                        iter::repeat(' ').take(TAB_LENGTH) // FIXME: selection should consider tabs
                    } else if c == '\n' {
                        iter::repeat(' ').take(1)
                    } else {
                        iter::repeat(c).take(1)
                    }
                })
                .collect::<String>();

            let ranges: Vec<(Style, &str)> = highlighter.highlight(&line_str, self.syntax_set);

            if i < line_offset {
                continue;
            }
            let y = i - line_offset;
            if y >= cmp::min(lines_height, line_count) {
                break;
            }

            // paint line number and initialize display for this line
            let line_index = line_offset + y;
            let line_number_style = Style {
                background: self.theme.settings.gutter.unwrap_or(Color {
                    r: 40,
                    g: 40,
                    b: 40,
                    a: 255,
                }),
                foreground: self.theme.settings.gutter_foreground.unwrap_or(Color {
                    r: 146,
                    g: 131,
                    b: 116,
                    a: 255,
                }),
                font_style: FontStyle::empty(),
            };
            self.screen
                .draw_with_style(0, y, line_number_style, &format!("{}", 1 + line_index));

            self.screen.draw_ranges(line_start, y, ranges);

            // draw selection over
            if let Some((selbeg, selend)) = content.sel() {
                let selection_style = Style {
                    foreground: self
                        .theme
                        .settings
                        .selection_foreground
                        .unwrap_or(Color::WHITE),
                    background: self.theme.settings.selection.unwrap_or(Color::BLACK),
                    font_style: FontStyle::empty(),
                };
                let beg = content.line_index_to_char_index(line_index);
                let end = beg + line_str.len() - 1;

                if *selbeg <= beg && *selend >= end {
                    // line is fully inside the selection
                    if line_str.is_empty() {
                        self.screen
                            .draw_with_style(line_start, y, selection_style, " ");
                    } else {
                        self.screen
                            .draw_with_style(line_start, y, selection_style, &line_str);
                    }
                } else if *selbeg >= beg && *selbeg <= end && *selend <= end && *selend >= beg {
                    // selection is inside the line
                    self.screen.draw_with_style(
                        line_start + (*selbeg - beg),
                        y,
                        selection_style,
                        &line_str[(*selbeg - beg)..=(*selend - beg)],
                    );
                } else if *selbeg <= end && *selbeg >= beg {
                    // line contains the beginning of the selection
                    self.screen.draw_with_style(
                        line_start + (*selbeg - beg),
                        y,
                        selection_style,
                        &line_str[(*selbeg - beg)..],
                    );
                } else if *selend <= end && *selend >= beg {
                    // line contains the end of the selection
                    self.screen.draw_with_style(
                        line_start,
                        y,
                        selection_style,
                        &line_str[..=(*selend - beg)],
                    );
                }
            }
        }
    }

    fn cursor_pos<T: Editable>(&self, content: &T) -> (usize, usize) {
        // TODO: column offsetting for long lines
        let line = content.line();
        let first_line = self.line_offset;
        let y = line - first_line as usize;
        // we can't trust the actual column because tabs have variable length
        let visual_col = content.col();
        let column: usize = content
            .iter_line(line)
            .map(|x| if x == '\t' { TAB_LENGTH } else { 1 })
            .take(visual_col)
            .sum();
        (
            (self.line_number_width(content.line_count()) as usize + 1 + column),
            y,
        )
    }

    fn line_number_width(&self, line_count: usize) -> u16 {
        line_count.to_string().len() as u16
    }

    fn status_height(&self) -> u16 {
        2
    }

    pub fn lines_height(&self) -> usize {
        let (_, screen_height) = terminal_size().unwrap();
        let incompressible = self.status_height() as usize;
        cmp::max(screen_height as usize, incompressible) - incompressible
    }

    pub fn lines_width(&self, line_count: usize) -> usize {
        let (screen_width, _) = terminal_size().unwrap();
        let incompressible = self.line_number_width(line_count) as usize + 1;
        cmp::max(screen_width as usize, incompressible) - incompressible
    }
}
