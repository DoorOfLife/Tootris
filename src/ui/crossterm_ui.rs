use std::borrow::BorrowMut;
use std::time::Duration;
use crossterm::event::{Event, KeyCode, poll, read};

use crate::game::tootris::{Communique, GameBroadcaster, GameMatrix, GameState, GameUpdateReceiver,
                           Master2UICommunique, PlayerMove, Point, UI2MasterCommunique,
                           UI2RenderCommunique, UiCommand, UIHandler};

pub struct TermUI {
    pub to_master: Option<GameBroadcaster<UI2MasterCommunique>>,
    pub to_render: Option<GameBroadcaster<UI2RenderCommunique>>,
    pub from_master: Option<GameUpdateReceiver<Master2UICommunique>>,
    pub state: Option<GameState>,
    pub score: Option<usize>,
}

impl TermUI {
    fn controller_update(&mut self) -> bool {
        if self.from_master.is_none() {
            return false;
        }
        let receiver = self.from_master.as_mut().unwrap().receiver.borrow_mut();
        let rec = receiver.try_recv();
        if rec.is_ok() {
            let com = rec.unwrap();
            if com.comm_type == Communique::Update {
                if com.state.is_some() {
                    self.state = com.state;
                }
                if com.score.is_some() {
                    self.score = com.score;
                }
                return true;
            }
        }
        return false;
    }

    fn send_controller_command(&mut self, typ: Communique, command: Option<UiCommand>,
                               player_move: Option<PlayerMove>) {
        if self.to_master.is_none() {
            return;
        }
        if self.to_master.as_mut().unwrap().channel_out.send(UI2MasterCommunique {
            comm_type: typ,
            command,
            player_move,
        }).is_err() {
            println!("failed to send controller command");
        }
    }

    fn send_render_command(&mut self, typ: Communique, command: Option<UiCommand>,
                           matrix: Option<GameMatrix>) {
        if self.to_render.is_none() {
            return;
        }

        if self.to_render.as_mut().unwrap().channel_out.send(UI2RenderCommunique {
            com_type: typ,
            matrix,
            command,
        }).is_err() {
            println!("failed to send render command");
        }
    }
}

impl UIHandler for TermUI {
    fn handle_ui(&mut self) -> bool {
        if self.controller_update() {
            self.send_render_command(Communique::Update, Some(UiCommand::Write(
                format!("Score:\n{}", self.score.unwrap()),
                Point { x: 20, y: 40 })), None);
        }

        let result = poll(Duration::from_millis(0));
        if result.is_err() {
            return false;
        }
        if result.unwrap() {
            let event = read();
            if event.is_err() {
                return false;
            }
            match event.unwrap() {
                Event::Key(event) => {
                    match event.code {
                        KeyCode::Enter => {}
                        KeyCode::Left => {
                            self.send_controller_command(Communique::Update, None, Some(PlayerMove::StepLeft));
                        }
                        KeyCode::Right => {
                            self.send_controller_command(Communique::Update, None, Some(PlayerMove::StepRight));
                        }
                        KeyCode::Up => {
                            self.send_controller_command(Communique::Update, None, Some(PlayerMove::RotateForward));
                        }
                        KeyCode::Down => {
                            self.send_controller_command(Communique::Update, None, Some(PlayerMove::StepDown));
                        }
                        KeyCode::Esc => {
                            self.send_controller_command(Communique::Update, Some(UiCommand::Exit), None);
                            self.send_render_command(Communique::Update, Some(UiCommand::Exit), None);
                        }
                        KeyCode::Backspace => {
                            if self.state.is_some() && self.state.as_ref().unwrap() == &GameState::Playing {
                                self.send_controller_command(Communique::Update, Some(UiCommand::Pause), None);
                            } else {
                                self.send_controller_command(Communique::Update, Some(UiCommand::Resume), None);
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        return true;
    }

    fn give_master_receiver(&mut self, receiver: GameUpdateReceiver<Master2UICommunique>) {
        self.from_master = Some(receiver);
    }

    fn give_master_broadcaster(&mut self, broadcaster: GameBroadcaster<UI2MasterCommunique>) {
        self.to_master = Some(broadcaster);
    }

    fn give_render_broadcaster(&mut self, broadcaster: GameBroadcaster<UI2RenderCommunique>) {
        self.to_render = Some(broadcaster);
    }
}