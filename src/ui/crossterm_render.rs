use std::borrow::{BorrowMut, Borrow};

use std::io::{Stdout, Write};
use crossterm::{
    cursor,
    QueueableCommand, style::{self}, terminal,
};
use crossterm::style::{Color, Styler};

use crossterm::terminal::ClearType;
use crate::game::tootris::{BlockColor, GameBlock, GameMatrix, GameState, GameUpdateReceiver, Master2RenderCommunique, Point, Renderer, UI2RenderCommunique, UiCommand};
use crate::game::tootris::ControllerCommand;
use terminal::Clear;
use style::{SetAttribute, Attribute, Print};
use cursor::MoveTo;
use crate::settings::{XRENDER_OFFSET, UI_ANCHOR};

pub struct TermRenderer {
    pub from_master: Option<GameUpdateReceiver<Master2RenderCommunique>>,
    pub from_ui: Option<GameUpdateReceiver<UI2RenderCommunique>>,
    pub out: Option<Stdout>,
    pub current_matrix: Option<GameMatrix>,
    pub draw_buffer: Option<GameMatrix>,
    pub term_size: Option<(u16, u16)>,
    pub state: Option<GameState>,
    pub render_offset: Option<Point>,
    pub ui_vector: Option<Vec<GameBlock>>,
}

impl TermRenderer {
    pub fn full_refresh(&mut self) {
        self.find_render_offset();
        terminal::enable_raw_mode().expect("It's raw mode or nothing babe");

        self.out.as_mut().unwrap().queue(Clear(ClearType::All)).expect("whatever");
        self.out.as_mut().unwrap().queue(cursor::DisableBlinking).expect("whatever2");
        self.out.as_mut().unwrap().queue(cursor::Hide).expect("whatever3");
        self.draw_whole_level();
        self.render_ui();
        self.out.as_mut().unwrap().flush().expect("The toilet is clogged.");
        self.term_size = Some(terminal::size().unwrap());
    }

    fn find_render_offset(&mut self) {
        self.render_offset = Some(Point {
            x: XRENDER_OFFSET,
            y: 0,
        });
    }

    fn render_ui(&mut self) {
        if self.ui_vector.is_none() {
            return;
        }
        for i in 0..self.ui_vector.as_ref().unwrap().len() {
            self.draw_single(self.ui_vector.as_ref().unwrap().get(i).unwrap().clone(),
                             Point { x: UI_ANCHOR.x, y: UI_ANCHOR.y + i }, true);
        }
    }

    /// Returns true if thread should continue
    pub fn maybe_render(&mut self) -> bool {
        if self.out.is_none() || self.from_master.is_none() {
            return false;
        }

        if self.state.is_some() {
            match self.state.as_ref().unwrap() {
                GameState::Exit => {
                    return false;
                }
                _ => {}
            }
        }

        let should_draw = self.check_handle_master_updates();
        if !self.check_handle_ui_updates() {
            return false;
        }
        if self.check_if_window_changed() {
            self.full_refresh();
        } else if should_draw {
            self.draw_updates();
            self.out.as_mut().expect("no stdout?")
                .flush().expect("forgot to flush.");
        }
        return true;
    }

    fn check_if_window_changed(&mut self) -> bool {
        let size = Some(terminal::size().unwrap());
        if self.term_size != size {
            return true;
        }
        return false;
    }

    /// returns true if there were updates
    fn check_handle_master_updates(&mut self) -> bool {
        let receiver = self.from_master
            .as_mut()
            .unwrap().receiver.borrow_mut();

        let rec = receiver.try_recv();
        if rec.is_ok() {
            let com = rec.unwrap();
            if com.level.is_some() {
                self.update_matrix(com.level.unwrap());
                return true;
            }
            if com.state.is_some() {
                self.state = com.state;
            }

            if com.command.is_some() {
                match com.command.unwrap() {
                    ControllerCommand::FullRefresh => {
                        self.full_refresh();
                    }
                }
            }
        }
        return false;
    }

    fn draw_whole_level(&mut self) {
        if self.current_matrix.is_none() {
            return;
        }
        if self.out.is_none() {
            return;
        }
        self.out.as_mut().unwrap().queue(SetAttribute(Attribute::Reset))
            .expect("super whatever");

        self.draw_buffer = self.current_matrix.clone();
        self.draw_updates();
    }

    fn draw_updates(&mut self) {
        if self.out.is_none() {
            return;
        }
        if self.draw_buffer.is_some() {
            self.out.as_mut().unwrap().queue(SetAttribute(Attribute::Reset))
                .expect("NOOOOOOOOOOOOOOOOOOO!");

            for y in 0..self.draw_buffer.as_ref().unwrap().len() {
                for x in 0..self.draw_buffer.as_ref().unwrap()[0].len() {
                    self.draw_single(
                        self.draw_buffer.as_ref().unwrap()[y][x].clone(), Point { x, y }, false);
                }
            }
        }
    }

    fn draw_single(&mut self, block: GameBlock, p: Point, override_offset: bool) {
        if self.out.is_none() {
            return;
        }
        let offset: Point;
        if override_offset || self.render_offset.is_none() { offset = Point { x: 0, y: 0 }; } else { offset = self.render_offset.unwrap(); }

        let output = self.out.as_mut().unwrap();
        output.queue(MoveTo((p.x * 2 + offset.x) as u16, (p.y + offset.y) as u16))
            .expect("dosh-dosh.");

        match block {
            GameBlock::Filled(color) => {
                output.queue(style::PrintStyledContent(
                    "██".bold()
                        .with(Self::map_color(&color))))
                    .expect("ton-ton?");
            }
            GameBlock::Empty => {
                output.queue(Print("..")).expect("ton-ton.");
            }
            GameBlock::Indestructible => {
                output.queue(style::PrintStyledContent(
                    "██".bold()
                        .with(style::Color::Grey))).expect("ton-ka-ton");
            }
            GameBlock::String(val, color) => {
                output.queue(style::PrintStyledContent(
                    val.bold()
                        .with(Self::map_color(&color))))
                    .expect("ton-ton?");
            }
            _ => {}
        }
    }

    fn map_color(block_color: &BlockColor) -> Color {
        match block_color {
            BlockColor::Blue => { Color::Blue }
            BlockColor::Magenta => { Color::Magenta }
            BlockColor::Yellow => { Color::Yellow }
            BlockColor::Green => { Color::Green }
            BlockColor::Cyan => { Color::Cyan }
            BlockColor::White => { Color::White }
        }
    }

    fn update_matrix(&mut self, new_matrix: GameMatrix) {
        if self.draw_buffer.is_none() || self.current_matrix.is_none() {
            return;
        }

        let write = self.draw_buffer.as_mut().unwrap();
        let current = self.current_matrix.as_mut().unwrap();
        for y in 0..new_matrix.len() {
            for x in 0..new_matrix[0].len() {
                if current[y][x] == new_matrix[y][x] {
                    write[y][x] = GameBlock::None;
                } else {
                    write[y][x] = new_matrix[y][x].clone();
                    current[y][x] = new_matrix[y][x].clone();
                }
            }
        }
    }

    fn check_handle_ui_updates(&mut self) -> bool {
        if self.from_ui.is_some() {
            let rec = self.from_ui.as_mut().unwrap().receiver.try_recv();
            if rec.is_ok() {
                let com = rec.unwrap();
                if com.vector.is_some() {
                    self.ui_vector = com.vector;
                }
                if com.command.is_some() {
                    match com.command.unwrap() {
                        UiCommand::Exit => {return false;}
                        UiCommand::RenderOffset(_) => {}
                        UiCommand::RefreshUi => {
                            self.render_ui();
                        }
                        _ => {}
                    }
                }
            }
        }
        return true;
    }
}

impl Renderer for TermRenderer {
    fn render(&mut self) -> bool {
        return self.maybe_render();
    }

    fn give_master_receiver(&mut self, receiver: GameUpdateReceiver<Master2RenderCommunique>) {
        self.from_master = Some(receiver);
    }

    fn give_ui_receiver(&mut self, receiver: GameUpdateReceiver<UI2RenderCommunique>) {
        self.from_ui = Some(receiver);
    }
}