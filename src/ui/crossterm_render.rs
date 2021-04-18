use crate::tootris::{Renderer, GameUpdateReceiver, UI2RenderCommunique, Master2RenderCommunique,
                     GameMatrix, GameBlock, Point, BlockColor};
use crate::piece;
use std::fmt::Debug;
use std::io::{stdout, Write, Stdout};
use crossterm::{
    ExecutableCommand, QueueableCommand,
    terminal, cursor, style::{self, Colorize}, Result,
};
use crate::piece::Piece;
use std::sync::mpsc::Receiver;
use std::borrow::BorrowMut;
use crossterm::style::{Color, Styler};
use crossterm::terminal::ClearType;

pub struct TermRenderer {
    pub master_receiver: Option<GameUpdateReceiver<Master2RenderCommunique>>,
    pub ui_receiver: Option<GameUpdateReceiver<UI2RenderCommunique>>,
    pub out: Option<Stdout>,
    pub current_matrix: Option<GameMatrix>,
    pub draw_buffer: Option<GameMatrix>,
    pub term_size: Option<(u16, u16)>,
}

impl TermRenderer {
    pub fn full_refresh(&mut self) {
        terminal::enable_raw_mode();

        self.out.as_mut().unwrap().queue(terminal::Clear(ClearType::All));
        self.out.as_mut().unwrap().queue(cursor::DisableBlinking);
        self.out.as_mut().unwrap().queue(cursor::Hide);
        self.draw_everything();
        self.out.as_mut().unwrap().flush();
        self.term_size = Some(terminal::size().unwrap());
    }

    pub fn check_maybe_render(&mut self) {
        if self.out.is_none() || self.master_receiver.is_none() {
            return;
        }
        let should_draw = self.check_handle_master_updates();
        if self.check_if_window_changed() {
            self.full_refresh();
        }
        else if should_draw {
            self.draw_updates();
            self.out.as_mut().unwrap().flush();
        }
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
        let receiver = self.master_receiver.as_mut().unwrap().receiver.borrow_mut();
        let rec = receiver.try_recv();
        if rec.is_ok() {
            let com = rec.unwrap();
            if com.level.is_some() {
                self.update_matrix(com.level.unwrap());
                return true;
            }
        }
        return false;
    }

    fn draw_everything(&mut self) {
        if self.current_matrix.is_none() {
            return;
        }
        if self.out.is_none() {
            return;
        }
        self.out.as_mut().unwrap().queue(style::SetAttribute(style::Attribute::Reset));
        self.draw_buffer = self.current_matrix.clone();
        self.draw_updates();
    }

    fn draw_updates(&mut self) {
        if self.out.is_none() {
            return;
        }
        if self.draw_buffer.is_some() {
            self.out.as_mut().unwrap().queue(style::SetAttribute(style::Attribute::Reset));
            for y in 0..self.draw_buffer.as_ref().unwrap().len() {
                for x in 0..self.draw_buffer.as_ref().unwrap()[0].len() {
                    self.draw_single(self.draw_buffer.as_ref().unwrap()[y][x].clone(), Point { x, y });
                }
            }
        }
    }

    fn draw_single(&mut self, block: GameBlock, p: Point) {
        if self.out.is_none() {
            return;
        }
        let output = self.out.as_mut().unwrap();
        output.queue(cursor::MoveTo((p.x * 2) as u16, p.y as u16));

        match block {
            GameBlock::Filled(_) => {
                output.queue(style::PrintStyledContent(
                    "██".bold()
                        .with(Self::map_color(block.get_color().unwrap()))));
            }
            GameBlock::Empty => {
                output.queue(style::Print(".."));
            }
            GameBlock::Indestructible => {
                output.queue(style::PrintStyledContent(
                    "██".bold()
                        .with(style::Color::Grey)));
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
            BlockColor::Undefined => { Color::Black }
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
                } else if current[y][x] != GameBlock::Indestructible {
                    write[y][x] = new_matrix[y][x].clone();
                    current[y][x] = new_matrix[y][x].clone();
                }
            }
        }
    }

    /// returns true if a full update is needed
    fn check_handle_ui_updates(&mut self) -> bool {
        return false;
    }
}

impl Renderer for TermRenderer {
    fn give_master_receiver(&mut self, receiver: GameUpdateReceiver<Master2RenderCommunique>) {
        self.master_receiver = Some(receiver);
    }

    fn give_ui_receiver(&mut self, receiver: GameUpdateReceiver<UI2RenderCommunique>) {
        self.ui_receiver = Some(receiver);
    }
}