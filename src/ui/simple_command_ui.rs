use crate::game::tootris::{UIHandler, UI2MasterCommunique, GameBroadcaster, GameUpdateReceiver,
                           Master2UICommunique, UI2RenderCommunique, PlayerMove, Communique};
use std::fmt::Debug;
use std::str;
use crate::ui::settings::{MOVE_LEFT_COMMAND, MOVE_RIGHT_COMMAND, ROT_DOWN_COMMAND, ROT_UP_COMMAND,
                          ROT_LEFT_COMMAND, ROT_RIGHT_COMMAND};


pub struct SimpleCommandUi {
    pub(crate) master_receiver: Option<GameUpdateReceiver<Master2UICommunique>>,
    pub(crate) master_broadcaster: Option<GameBroadcaster<UI2MasterCommunique>>,
    pub(crate) command_queue: Vec<PlayerMove>,

}

impl SimpleCommandUi {
    pub fn submit_command(&mut self, command: &str) {
        let split = command.split(",");
        for s in split {
            let mov = Self::command_to_player_move(s);
            if mov.is_some() {
                self.send_input_to_master(mov.unwrap());
            }
        }
    }
    fn command_to_player_move(command: &str) -> Option<PlayerMove> {
        if Self::command_matches_csv(command, MOVE_LEFT_COMMAND) {
            return Some(PlayerMove::StepLeft);
        }
        if Self::command_matches_csv(command, MOVE_RIGHT_COMMAND) {
            return Some(PlayerMove::StepRight);
        }
        if Self::command_matches_csv(command, ROT_DOWN_COMMAND) {
            return Some(PlayerMove::OrientDown);
        }
        if Self::command_matches_csv(command, ROT_UP_COMMAND) {
            return Some(PlayerMove::OrientUp);
        }
        if Self::command_matches_csv(command, ROT_LEFT_COMMAND) {
            return Some(PlayerMove::OrientLeft);
        }
        if Self::command_matches_csv(command, ROT_RIGHT_COMMAND) {
            return Some(PlayerMove::OrientRight);
        }
        return None;
    }

    fn command_matches_csv(command: &str, csv: &str) -> bool {
        let split = csv.split(",");

        for s in split {
            if command == s {
                return true;
            }
        }
        return false;
    }

    fn send_input_to_master(&mut self, mov: PlayerMove) {
        if self.master_broadcaster.is_some() {
            let com = UI2MasterCommunique {
                comm_type: Communique::Update,
                state: None,
                player_move: Some(mov),
            };
            self.master_broadcaster.as_mut().unwrap().channel_out.send(com);
        }
    }
}

impl UIHandler for SimpleCommandUi {
    fn process_input(&mut self) {
        for mov in self.command_queue.clone() {
            self.send_input_to_master(mov);
        }
        self.command_queue.clear();
    }

    fn give_master_receiver(&mut self, receiver: GameUpdateReceiver<Master2UICommunique>) {
        self.master_receiver = Some(receiver);
    }

    fn give_master_broadcaster(&mut self, broadcaster: GameBroadcaster<UI2MasterCommunique>) {
        self.master_broadcaster = Some(broadcaster);
    }

    fn give_render_broadcaster(&mut self, broadcaster: GameBroadcaster<UI2RenderCommunique>) {
        todo!()
    }
}