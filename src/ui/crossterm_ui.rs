use crossterm::event::{poll, read, Event, KeyCode, KeyEvent};

use std::fmt::Debug;
use std::str;

use std::time::Duration;

use crate::game::tootris::{UIHandler, UI2MasterCommunique, GameBroadcaster, GameUpdateReceiver, Master2UICommunique, UI2RenderCommunique, PlayerMove, Communique, UiCommand, GameMatrix};
use crate::ui::settings::{MOVE_LEFT_COMMAND, MOVE_RIGHT_COMMAND, ROT_DOWN_COMMAND, ROT_UP_COMMAND,
                          ROT_LEFT_COMMAND, ROT_RIGHT_COMMAND};

pub struct TermUI {
    pub to_master: Option<GameBroadcaster<UI2MasterCommunique>>,
    pub to_render: Option<GameBroadcaster<UI2RenderCommunique>>,
    pub from_master: Option<GameUpdateReceiver<Master2UICommunique>>,
}

impl TermUI {
    fn send_controller_command(&mut self, typ: Communique, command: Option<UiCommand>,
                               player_move: Option<PlayerMove>) {
        if self.to_master.is_none() {
            return;
        }
        self.to_master.as_mut().unwrap().channel_out.send(UI2MasterCommunique {
            comm_type: typ,
            command,
            player_move,
        });
    }

    fn send_render_command(&mut self, typ: Communique, command: Option<UiCommand>,
                           matrix: Option<GameMatrix>) {
        if self.to_render.is_none() {
            return;
        }

        self.to_render.as_mut().unwrap().channel_out.send(UI2RenderCommunique {
            com_type: typ,
            matrix,
            command,
        });
    }
}

impl UIHandler for TermUI {
    fn handle_ui(&mut self) -> bool {
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
                        _ => {}
                    }
                }
                Event::Resize(_, _) => {}
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