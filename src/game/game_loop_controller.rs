extern crate stopwatch;

use rand::{Rng, thread_rng};
use stopwatch::Stopwatch;

use crate::game::piece::{Piece};
use crate::game::piece_types::{PieceDefinition, PieceDefinitions, PieceFreezeProperty};

use crate::game::tootris::{BlockColor, Communique, Controller, GameBlock, GameBroadcaster, GameMatrix, GameState, GameUpdateReceiver, Master2RenderCommunique, Master2UICommunique, PlayerMove, Point, Rotation, UI2MasterCommunique, UiCommand, ControllerCommand};
use crate::game::tootris::Communique::Update;
use crate::settings::*;
use crate::game::tootris::GameState::{Tootris, Exit};
use std::ops::Mul;

pub struct EvilGameMaster {
    pub level: GameMatrix,
    pub gom: Option<GameMatrix>,
    pub completed_rows: Vec<usize>,
    pub active_piece: Option<Piece>,
    speed: usize,
    pub score: usize,
    pub num_pieces: usize,
    piece_map: PieceDefinitions,
    piece_bucket: Vec<Piece>,
    pub sw: Stopwatch,
    pub state: GameState,
    pub render_slave: Option<GameBroadcaster<Master2RenderCommunique>>,
    pub ui_slave: Option<GameBroadcaster<Master2UICommunique>>,
    pub ui_listener: Option<GameUpdateReceiver<UI2MasterCommunique>>,
}

impl Controller for EvilGameMaster {
    fn process(&mut self) {
        self.process_game();
    }

    fn give_ui_broadcaster(&mut self, broadcaster: GameBroadcaster<Master2UICommunique>) {
        self.ui_slave = Some(broadcaster);
    }

    fn give_render_broadcaster(&mut self, broadcaster: GameBroadcaster<Master2RenderCommunique>) {
        self.render_slave = Some(broadcaster);
    }

    fn give_ui_receiver(&mut self, receiver: GameUpdateReceiver<UI2MasterCommunique>) {
        self.ui_listener = Some(receiver);
    }
}

impl EvilGameMaster {
    pub fn create_level(x: usize, y: usize) -> Vec<Vec<GameBlock>> {
        let mut level: Vec<Vec<GameBlock>> = Vec::new();
        for _rows in 0..y {
            level.push(Self::create_empty_row(x));
        }
        return level;
    }

    pub fn create_empty_row(x: usize) -> Vec<GameBlock> {
        let mut row: Vec<GameBlock> = Vec::new();
        row.push(GameBlock::Indestructible);
        for _point in 1..x - 1 {
            row.push(GameBlock::Empty);
        }
        row.push(GameBlock::Indestructible);
        return row;
    }
    pub fn new(height: usize, width: usize, initial_piece: Option<Piece>,
               render_slave: Option<GameBroadcaster<Master2RenderCommunique>>,
               ui_slave: Option<GameBroadcaster<Master2UICommunique>>,
               ui_listener: Option<GameUpdateReceiver<UI2MasterCommunique>>) -> Self {
        let mut s = EvilGameMaster {
            level: Self::create_level(width, height),
            completed_rows: Vec::new(),
            active_piece: initial_piece,
            speed: 1,
            score: 0,
            num_pieces: 0,
            piece_map: PieceDefinitions::new(),
            piece_bucket: Vec::with_capacity(OPTION_BUCKET_MAX_SIZE),
            sw: Stopwatch::new(),
            state: GameState::Start,
            render_slave,
            ui_slave,
            ui_listener,
            gom: None,
        };
        s.create_level_boundaries();
        return s;
    }
    fn next_piece(&mut self) -> bool {
        if self.piece_bucket.len() < OPTION_BUCKET_MINIMUM_SIZE {
            self.fill_piece_bucket();
        }
        self.active_piece = self.piece_bucket.pop();
        if self.is_xcolliding(self.active_piece.as_ref().unwrap(), None) ||

            self.ycolliding(self.active_piece.as_ref().unwrap(), None) {
            return false;
        }
        return true;
    }

    fn fill_piece_bucket(&mut self) {
        for _ in self.piece_bucket.len()..OPTION_BUCKET_MAX_SIZE {
            self.piece_bucket.push(
                Piece::of_type(self.random_piece_type(),
                               Self::random_color(),
                               Point { y: 1, x: self.level[0].len() / 2 }));
        }
    }

    fn random_color() -> BlockColor {
        let rand_num = thread_rng().gen_range(0..=5);
        match rand_num {
            0 => BlockColor::Blue,
            1 => BlockColor::Green,
            2 => BlockColor::Cyan,
            3 => BlockColor::White,
            4 => BlockColor::Magenta,
            5 => BlockColor::Yellow,
            _ => panic!("random color out of range")
        }
    }

    fn random_piece_type(&self) -> &PieceDefinition {
        let rand_num = thread_rng().gen_range(0..=6);
        match rand_num {
            0 => self.piece_map.get_piece_def(PIECE_LINE),
            1 => self.piece_map.get_piece_def(PIECE_SQUARE),
            2 => self.piece_map.get_piece_def(PIECE_PODIUM),
            3 => self.piece_map.get_piece_def(PIECE_L),
            4 => self.piece_map.get_piece_def(PIECE_J),
            5 => self.piece_map.get_piece_def(PIECE_S),
            6 => self.piece_map.get_piece_def(PIECE_Z),
            _ => panic!("random piece out of range")
        }
    }

    pub fn process_game(&mut self) -> bool {
        let mut should_update_render = false;
        let mut should_continue = true;

        match self.state {
            GameState::Playing => {
                if self.active_piece.is_none() {
                    if !self.next_piece() {
                        self.state = GameState::End;
                        self.active_piece = None;
                        return should_continue;
                    }
                }
                if self.next_tick() {
                    should_update_render = true;

                    if !self.advance_active_piece() {
                        self.freeze_active_piece();
                        if !self.completed_rows.is_empty() {
                            self.state = Tootris;
                        }
                        //in this case, return to immediately "instantiate" next piece
                        //or handle the tootris state
                        return should_continue; //in the next loop cycle
                    }
                }
                //todo: check UI input
                if self.process_input_commands() {
                    should_update_render = true;
                }
            }
            GameState::Tootris => {
                if self.next_tick() {
                    self.send_state_to_ui();
                    self.score += (self.completed_rows.len() * self.level[0].len())
                        .mul(self.completed_rows.len());

                    let mut new_matrix: GameMatrix = Vec::with_capacity(self.level.len());
                    if self.speed < 99 - self.completed_rows.len() {
                        self.speed+= self.completed_rows.len();
                    } else {
                        self.speed = 99;
                    }

                    //in new matrix, create new empty rows at the top
                    for _new_row in 0..self.completed_rows.len() {
                        new_matrix.push(Self::create_empty_row(self.level[0].len()))
                    }
                    //in new matrix, add the remaining rows from the old
                    for i in 0..self.level.to_owned().len() {
                        if self.completed_rows.contains(&i) {
                            continue;
                        }
                        new_matrix.push(self.level[i].to_owned());
                    }
                    self.level = new_matrix;
                    self.completed_rows = Vec::new();
                    self.state = GameState::Playing;
                    should_update_render = true;
                    self.send_state_to_ui();
                }
            }

            GameState::Paused => {
                self.process_input_commands();
            }
            GameState::End => {
                if !self.sw.is_running() {
                    self.sw.start();
                }
                if self.next_tick() {
                    if self.active_piece.is_none() {
                        self.active_piece = Some(Piece
                        ::new(PieceDefinitions::new().get_piece_def(
                            GAME_OVER_PIECE.as_ref()).def.to_owned(),
                              PieceFreezeProperty::Normal,
                              BlockColor::Magenta, Point { x: 5, y: self.level.len() - 1 }));
                    }
                    self.send_render_update(Some(ControllerCommand::FullRefresh));
                }
                self.process_input_commands();
            }
            GameState::Start => {
                self.process_input_commands();
            }
            GameState::Exit => {
                should_continue = false;
            }
        }
        if should_update_render {
            self.send_render_update(None);
        }
        return should_continue;
    }

    fn process_input_commands(&mut self) -> bool {
        if self.ui_listener.is_some() {
            let mut command = self.ui_listener.as_mut().unwrap().receiver.try_recv();
            if command.is_err() {
                return false;
            }
            if command.as_ref().unwrap().is_command() {
                match command.as_mut().unwrap().command.as_ref().unwrap() {
                    UiCommand::New => {
                        self.new_game();
                    }
                    UiCommand::Pause => {
                        self.pause_game();
                    }
                    UiCommand::Resume => {
                        self.resume_game();
                    }
                    UiCommand::Exit => {
                        self.state = Exit;
                        self.send_render_update(None);
                    }
                    _ => {}
                }
                return true;
            }

            if command.as_ref().unwrap().is_player_move() {
                return self.process_move(command.as_ref().unwrap().player_move.as_ref().unwrap());
            }
            return false;
        }
        return false;
    }

    fn next_tick(&mut self) -> bool {
        let tick_ms = ((100 as usize - self.speed) * OPTION_TICK_BASE_MS) as i64;
        if tick_ms <= self.sw.elapsed_ms() {
            self.sw.restart();
            return true;
        }
        return false;
    }

    pub fn exit(&mut self) {
        if self.sw.is_running() {
            self.sw.stop();
        }
        self.state = GameState::Exit;
    }

    pub fn new_game(&mut self) {
        if self.sw.is_running() {
            self.sw.stop();
        }
        self.sw.reset();
        self.score = 0;
        self.active_piece = None;
        self.level = Self::create_level(self.level[0].len(), self.level.len());
        self.create_level_boundaries();
        self.state = GameState::Start;
        self.send_state_to_ui();
    }

    pub fn resume_game(&mut self) {
        if !self.sw.is_running() {
            self.sw.start();
        }
        self.state = GameState::Playing;
        self.send_state_to_ui();
    }

    pub fn pause_game(&mut self) {
        if self.sw.is_running() {
            self.sw.stop();
        }
        self.state = GameState::Paused;
        self.send_state_to_ui();
        return;
    }

    pub fn give_render_slave(&mut self, broadcaster: GameBroadcaster<Master2RenderCommunique>) {
        self.render_slave = Some(broadcaster);
    }

    pub fn give_ui_slave(&mut self, broadcaster: GameBroadcaster<Master2UICommunique>) {
        self.ui_slave = Some(broadcaster);
    }

    pub fn give_ui_receiver(&mut self, receiver: GameUpdateReceiver<UI2MasterCommunique>) {
        self.ui_listener = Some(receiver);
    }

    fn freeze_active_piece(&mut self) {
        self.active_piece.as_ref().unwrap().place_in_matrix(self.level.as_mut_slice());
        self.active_piece = None;
        self.find_completed_rows();
    }

    fn find_completed_rows(&mut self) {
        'rows: for row in 0..self.level.len() {
            let mut begin = false;
            let mut end = false;

            for cell in self.level[row].as_slice() {
                match cell {
                    &GameBlock::Indestructible => {
                        if end {
                            //this must be a border row
                            continue 'rows;
                        }
                        if begin { //If it's not the beginning it must be the end
                            end = true;
                        } else { //If it's not the end then it must be the beginning
                            begin = true;
                        }
                    }
                    &GameBlock::Filled(_) => {
                        if end {
                            self.ui_warn("Invalid state: piece block appears after border");
                        } else if begin {
                            //seems legit
                        } else {
                            self.ui_warn("Invalid state: piece block appears before border");
                        }
                    }
                    _ => {
                        continue 'rows;
                    }
                }
            }
            //If we get here, the row has only filled blocks
            if begin && end {
                self.completed_rows.push(row);
            }
        }
    }

    fn ui_warn(&self, msg: &'static str) {
        if self.ui_slave.is_none() {
            eprintln!("No ui slave!");
        }
        let result = self.ui_slave.as_ref().unwrap().channel_out.send(Master2UICommunique {
            comm_type: Communique::Info(msg),
            state: None,
            score: None,
        });

        if result.is_err() {
            eprintln!("Could not send message to ui slave");
        }
    }

    fn is_xcolliding(&self, piece: &Piece, point_override: Option<&Point>) -> bool {
        return self.is_point_colliding(piece.find_xboundaries(point_override));
    }

    fn ycolliding(&self, piece: &Piece, point_override: Option<&Point>) -> bool {
        return self.is_point_colliding(piece.find_yboundaries(point_override));
    }

    pub(crate) fn is_point_colliding(&self, bounds: Vec<Point>) -> bool {
        for point in bounds {
            if point.y >= self.level.len() {
                return true;
            }
            if point.x >= self.level[0].len() {
                return true;
            }
            if self.level[point.y][point.x] != GameBlock::Empty {
                return true;
            }
        }
        return false;
    }

    /**
     * returns true if the active_piece was successfully advanced, false if not.
     */
    fn advance_active_piece(&mut self) -> bool {
        if self.active_piece.is_some() {
            return self.vertical_move(1);
        }
        return false;
    }

    pub fn process_move(&mut self, mov: &PlayerMove) -> bool {
        match mov {
            PlayerMove::RotateForward => self.rotate_active_piece(&Rotation::Forward),
            PlayerMove::StepLeft => self.horizontal_move(1, true),
            PlayerMove::StepRight => self.horizontal_move(1, false),
            PlayerMove::StepDown => self.vertical_move(1),
            _ => false,
        }
    }

    fn vertical_move(&mut self, amount: usize) -> bool {
        if self.active_piece.is_some() {
            let mut point = self.active_piece.as_ref().unwrap().location.clone();
            point.y += amount;
            self.active_piece.as_mut().unwrap().move_to(point);
            let bounds = self.active_piece.as_ref().unwrap().points(None);
            if self.is_point_colliding(bounds) {
                self.active_piece.as_mut().unwrap().rollback_move();
                return false;
            }
        }
        true
    }

    fn horizontal_move(&mut self, amount: usize, reverse: bool) -> bool {
        if self.active_piece.is_some() {
            let mut point = self.active_piece.as_ref().unwrap().location.clone();
            if reverse { point.x -= amount; } else { point.x += amount; }
            self.active_piece.as_mut().unwrap().move_to(point);
            if self.is_point_colliding(self.active_piece.as_ref().unwrap().points(None)) {
                self.active_piece.as_mut().unwrap().rollback_move();
                return false;
            }
        }
        return true;
    }

    /**
    * Returns true if active_piece was successfully rotated forward, false if not
    */
    fn rotate_active_piece(&mut self, rot: &Rotation) -> bool {
        if self.active_piece.is_some() {
            self.active_piece.as_mut().unwrap().rotate(rot);

            if self.is_point_colliding(self.active_piece.as_ref().unwrap().points(None)) {
                self.active_piece.as_mut().unwrap().rollback_rotation();
                return false;
            }
        }
        true
    }

    fn create_level_boundaries(&mut self) {
        for y in 0..self.level.len() {
            for x in 0..self.level[y].len() {
                //Fill in a frame around the level matrix, not the top
                if y == self.level.len() - 1 || x == 0 || x == self.level[y].len() - 1 {
                    self.level[y][x] = GameBlock::Indestructible;
                } else {
                    self.level[y][x] = GameBlock::Empty;
                }
            }
        }
    }

    fn send_state_to_ui(&mut self) -> bool {
        if self.ui_slave.is_some() {
            return self.ui_slave.as_ref().unwrap().channel_out.send(Master2UICommunique {
                comm_type: Update,
                state: Some(self.state.clone()),
                score: Some(self.score),
            }).is_err();
        }
        return false;
    }

    fn send_render_update(&mut self, command: Option<ControllerCommand>) -> bool {
        if self.render_slave.is_some() {
            let mut level_update = self.level.clone();
            if self.active_piece.is_some() {
                self.active_piece.as_ref().unwrap().place_in_matrix(level_update.as_mut_slice());
            }

            return self.render_slave.as_ref().unwrap().channel_out.send(Master2RenderCommunique {
                comm_type: Communique::Update,
                level: Some(level_update),
                state: Some(self.state.clone()),
                score: Some(self.score),
                command,
            }).is_err();
        }
        return false;
    }
}
