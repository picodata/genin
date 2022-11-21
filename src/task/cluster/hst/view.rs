#![allow(dead_code)]

use std::borrow::Cow;
use tabled::{papergrid::AnsiColor, Alignment};

use crate::task::cluster::name::Name;

pub const FG_BLUE: AnsiColor =
    AnsiColor::new(Cow::Borrowed("\u{1b}[34m"), Cow::Borrowed("\u{1b}[39m"));
pub const FG_BRIGHT_BLACK: AnsiColor =
    AnsiColor::new(Cow::Borrowed("\u{1b}[90m"), Cow::Borrowed("\u{1b}[39m"));
pub const FG_BRIGHT_BLUE: AnsiColor =
    AnsiColor::new(Cow::Borrowed("\u{1b}[94m"), Cow::Borrowed("\u{1b}[39m"));
pub const FG_BRIGHT_CYAN: AnsiColor =
    AnsiColor::new(Cow::Borrowed("\u{1b}[96m"), Cow::Borrowed("\u{1b}[39m"));
pub const FG_BRIGHT_GREEN: AnsiColor =
    AnsiColor::new(Cow::Borrowed("\u{1b}[92m"), Cow::Borrowed("\u{1b}[39m"));
pub const FG_BRIGHT_MAGENTA: AnsiColor =
    AnsiColor::new(Cow::Borrowed("\u{1b}[95m"), Cow::Borrowed("\u{1b}[39m"));
pub const FG_BRIGHT_RED: AnsiColor =
    AnsiColor::new(Cow::Borrowed("\u{1b}[91m"), Cow::Borrowed("\u{1b}[39m"));
pub const FG_BRIGHT_WHITE: AnsiColor =
    AnsiColor::new(Cow::Borrowed("\u{1b}[97m"), Cow::Borrowed("\u{1b}[39m"));
pub const FG_BRIGHT_YELLOW: AnsiColor =
    AnsiColor::new(Cow::Borrowed("\u{1b}[93m"), Cow::Borrowed("\u{1b}[39m"));
pub const FG_CYAN: AnsiColor =
    AnsiColor::new(Cow::Borrowed("\u{1b}[36m"), Cow::Borrowed("\u{1b}[39m"));
pub const FG_GREEN: AnsiColor =
    AnsiColor::new(Cow::Borrowed("\u{1b}[32m"), Cow::Borrowed("\u{1b}[39m"));
pub const FG_MAGENTA: AnsiColor =
    AnsiColor::new(Cow::Borrowed("\u{1b}[35m"), Cow::Borrowed("\u{1b}[39m"));
pub const FG_RED: AnsiColor =
    AnsiColor::new(Cow::Borrowed("\u{1b}[31m"), Cow::Borrowed("\u{1b}[39m"));
pub const FG_WHITE: AnsiColor =
    AnsiColor::new(Cow::Borrowed("\u{1b}[37m"), Cow::Borrowed("\u{1b}[39m"));
pub const FG_YELLOW: AnsiColor =
    AnsiColor::new(Cow::Borrowed("\u{1b}[33m"), Cow::Borrowed("\u{1b}[39m"));
pub const BG_BLACK: AnsiColor =
    AnsiColor::new(Cow::Borrowed("\u{1b}[40m"), Cow::Borrowed("\u{1b}[49m"));
pub const BG_BLUE: AnsiColor =
    AnsiColor::new(Cow::Borrowed("\u{1b}[44m"), Cow::Borrowed("\u{1b}[49m"));
pub const BG_BRIGHT_BLACK: AnsiColor =
    AnsiColor::new(Cow::Borrowed("\u{1b}[100m"), Cow::Borrowed("\u{1b}[49m"));
pub const BG_BRIGHT_BLUE: AnsiColor =
    AnsiColor::new(Cow::Borrowed("\u{1b}[104m"), Cow::Borrowed("\u{1b}[49m"));
pub const BG_BRIGHT_CYAN: AnsiColor =
    AnsiColor::new(Cow::Borrowed("\u{1b}[106m"), Cow::Borrowed("\u{1b}[49m"));
pub const BG_BRIGHT_GREEN: AnsiColor =
    AnsiColor::new(Cow::Borrowed("\u{1b}[102m"), Cow::Borrowed("\u{1b}[49m"));
pub const BG_BRIGHT_MAGENTA: AnsiColor =
    AnsiColor::new(Cow::Borrowed("\u{1b}[105m"), Cow::Borrowed("\u{1b}[49m"));
pub const BG_BRIGHT_RED: AnsiColor =
    AnsiColor::new(Cow::Borrowed("\u{1b}[101m"), Cow::Borrowed("\u{1b}[49m"));
pub const BG_BRIGHT_WHITE: AnsiColor =
    AnsiColor::new(Cow::Borrowed("\u{1b}[107m"), Cow::Borrowed("\u{1b}[49m"));
pub const BG_BRIGHT_YELLOW: AnsiColor =
    AnsiColor::new(Cow::Borrowed("\u{1b}[103m"), Cow::Borrowed("\u{1b}[49m"));
pub const BG_CYAN: AnsiColor =
    AnsiColor::new(Cow::Borrowed("\u{1b}[46m"), Cow::Borrowed("\u{1b}[49m"));
pub const BG_GREEN: AnsiColor =
    AnsiColor::new(Cow::Borrowed("\u{1b}[42m"), Cow::Borrowed("\u{1b}[49m"));
pub const BG_MAGENTA: AnsiColor =
    AnsiColor::new(Cow::Borrowed("\u{1b}[45m"), Cow::Borrowed("\u{1b}[49m"));
pub const BG_RED: AnsiColor =
    AnsiColor::new(Cow::Borrowed("\u{1b}[41m"), Cow::Borrowed("\u{1b}[49m"));
pub const BG_WHITE: AnsiColor =
    AnsiColor::new(Cow::Borrowed("\u{1b}[47m"), Cow::Borrowed("\u{1b}[49m"));
pub const BG_YELLOW: AnsiColor =
    AnsiColor::new(Cow::Borrowed("\u{1b}[43m"), Cow::Borrowed("\u{1b}[49m"));

#[derive(Clone, Debug)]
pub struct View {
    pub color: AnsiColor<'static>,
    pub alignment: Alignment,
}

impl Default for View {
    fn default() -> Self {
        Self {
            color: Default::default(),
            alignment: Alignment::left(),
        }
    }
}

impl PartialEq for View {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl Eq for View {}

pub struct TableColors {
    name: Name,
    wheel: Vec<AnsiColor<'static>>,
}

impl TableColors {
    pub fn new() -> Self {
        Self {
            name: Name::from(""),
            wheel: vec![
                FG_WHITE,
                FG_BLUE,
                FG_CYAN,
                FG_GREEN,
                FG_MAGENTA,
                FG_RED,
                FG_YELLOW,
                FG_BRIGHT_WHITE,
                FG_BRIGHT_BLUE,
                FG_BRIGHT_CYAN,
                FG_BRIGHT_GREEN,
                FG_BRIGHT_MAGENTA,
                FG_BRIGHT_RED,
                FG_BRIGHT_YELLOW,
            ],
        }
    }

    pub fn next_color(&mut self, name: Name) -> AnsiColor<'static> {
        if !self.name.eq(&name) {
            self.wheel.rotate_left(1);
            self.name = name;
        }
        self.wheel.last().unwrap().clone()
    }
}
