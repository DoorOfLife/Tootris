extern crate stopwatch;

use stopwatch::Stopwatch;

use std::sync::mpsc::Sender;
use std::sync::mpsc::Receiver;
use rand::{thread_rng, Rng};

use std::any::Any;
use std::borrow::Borrow;

use crate::tootris::{Master2RenderCommunique, Master2UICommunique, GameState, Communique, GameBlock,
                     Rotation, PlayerMove, Point, UI2MasterCommunique, GameUpdateReceiver,
                     GameBroadcaster, GameMatrix, BlockColor};

use crate::piece_types::{DefinitionBlock, PieceDefinition,
                         PieceFreezeProperty, LINE, SQUARE, PODIUM, PieceDefinitions};

use crate::piece::{Piece, PieceState};
use crate::tootris::Communique::Update;
use crate::settings::*;


pub struct EvilGameMaster {
    pub level: GameMatrix,
    pub completed_rows: Vec<usize>,
    pub active_piece: Option<Piece>,
    speed: usize,
    pub score: usize,
    num_pieces: usize,
    piece_map: PieceDefinitions,
    piece_bucket: Vec<Piece>,
    pub sw: Stopwatch,
    slide_sw: Stopwatch,
    pub state: GameState,
    pub render_slave: Option<GameBroadcaster<Master2RenderCommunique>>,
    pub ui_slave: Option<GameBroadcaster<Master2UICommunique>>,
    pub ui_listener: Option<GameUpdateReceiver<UI2MasterCommunique>>,
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
        for _point in 0..x {
            row.push(GameBlock::Empty);
        }
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
            slide_sw: Stopwatch::new(),
            state: GameState::Start,
            render_slave,
            ui_slave,
            ui_listener,
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

    pub fn process_game(&mut self) {
        let mut should_update_render = false;

        match self.state {
            GameState::Playing => {
                if self.active_piece.is_none() {
                    if !self.next_piece() {
                        self.state = GameState::End;
                        return;
                    }
                }
                if self.next_tick() {
                    should_update_render = true;

                    if !self.advance_active_piece() {
                        println!("{}",self.active_piece.as_ref().unwrap());
                        self.freeze_active_piece();
                        //in this case, return to immediately "instantiate" next piece
                        return; //in the next loop cycle
                    }
                }

                match self.active_piece.as_ref().unwrap().piece_state {
                    PieceState::SlidingLeft => {
                        self.slide_if_time(true);
                    }
                    PieceState::SlidingRight => {
                        self.slide_if_time(false);
                    }
                    PieceState::Falling => {
                        self.fall_if_time();
                    }
                    _ => {}
                }
                //todo: check UI input
                if self.process_input_commands() {
                    should_update_render = true;
                }
                //if new movement:
                //should_update_render = true;
            }

            GameState::PieceFreeze => {
                self.send_state_to_ui(GameState::PieceFreeze);
            }

            GameState::Tootris => {
                if self.next_tick() {
                    self.send_state_to_ui(GameState::Tootris);
                    self.score += (self.completed_rows.len() * self.level[0].len())
                        .pow(self.completed_rows.len() as u32);

                    let mut new_matrix: GameMatrix = Vec::with_capacity(self.level.len());

                    //in new matrix, create new empty rows at the top
                    for _new_row in 0..self.completed_rows.len() {
                        new_matrix.push(Self::create_empty_row(self.level[0].len()))
                    }

                    //in old matrix, remove the completed rows
                    for remove_row in self.completed_rows.to_owned() {
                        self.level.remove(remove_row);
                    }

                    //in new matrix, add the remaining rows from the old
                    for mut remaning_row in self.level.to_owned() {
                        new_matrix.push(remaning_row);
                    }
                    self.level = new_matrix;
                    self.state = GameState::Playing;
                    self.send_state_to_ui(GameState::Playing);
                }
            }

            GameState::Pause => {
                self.sw.stop();
                self.state = GameState::Paused;
                self.send_state_to_ui(GameState::Paused);
                return;
            }

            GameState::Paused => {
                //todo: Check for input to unpause
            }
            GameState::End => {
                self.send_state_to_ui(GameState::End);
                self.send_render_update(GameState::End);
                self.game_over();
                //todo: implement ending logic
            }
            GameState::Start => {
                self.sw.start();
                self.state = GameState::Playing;
                self.send_state_to_ui(GameState::Playing);
                return;
            }
            GameState::Reset => {
                if self.sw.is_running() {
                    self.sw.stop();
                }
                self.sw.reset();
                //todo: reset level-data
                self.state = GameState::Paused;
                self.send_state_to_ui(GameState::Paused);
                return;
            }
        }
        if should_update_render {
            self.send_render_update(self.state);
        }
    }


    fn process_input_commands(&mut self) -> bool {
        if self.ui_listener.is_some() {
            let mut terminate = false;
            let mut command = self.ui_listener.as_mut().unwrap().receiver.try_recv();
            if command.is_err() {
                return false;
            }
            if command.as_ref().unwrap().state.is_some() {
                terminate = self.handle_state_request(command.as_ref().unwrap().state.as_ref().unwrap());
            }
            if terminate { return false; };
            if command.as_ref().unwrap().player_move.is_some() {
                return self.process_move(command.as_ref().unwrap().player_move.as_ref().unwrap());
            }
            return false;
        }
        return false;
    }


    /**
    * returns true if ui processing should continue after handling state requests
    */
    fn handle_state_request(&mut self, state: &GameState) -> bool {
        match state {
            GameState::End => {
                self.state = GameState::End;
                return false;
            }
            GameState::Reset => {
                self.state = GameState::Reset;
                return false;
            }
            _ => {}
        }
        match self.state {
            GameState::Playing => {
                match state {
                    GameState::Pause => {
                        self.pause_game();
                        return false;
                    }
                    _ => {}
                }
            }
            GameState::Paused => {
                match state {
                    GameState::Start => {
                        self.resume_game();
                        return false;
                    }

                    _ => {}
                }
            }
            GameState::End => {
                match state {
                    GameState::Start => {
                        self.new_game();
                        return false;
                    }

                    _ => {}
                }
            }
            _ => {}
        }
        return true;
    }

    fn fall_if_time(&mut self) -> bool {
        if self.slide_sw.elapsed_ms() >= OPTION_FALL_INTERVAL_MS {
            self.vertical_move(1);
            self.slide_sw.restart();
            return true;
        }
        return false;
    }

    fn slide_if_time(&mut self, reverse: bool) -> bool {
        if self.slide_sw.elapsed_ms() >= OPTION_SLIDE_INTERVAL_MS {
            self.horizontal_move(1, reverse);
            self.slide_sw.restart();
            return true;
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

    pub fn new_game(&mut self) {
        self.score = 0;
        self.active_piece = None;
        self.level = Self::create_level(self.level[0].len(), self.level.len());
        self.create_level_boundaries();
        self.state = GameState::Start;
    }

    pub fn resume_game(&mut self) {
        self.state = GameState::Start;
    }

    pub fn pause_game(&mut self) {
        self.state = GameState::Pause;
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
        //todo: place piece in level, destroy piece, check for complete rows
        self.active_piece.as_ref().unwrap().place_in_matrix(self.level.as_mut_slice());
        self.active_piece = None;
        self.find_completed_rows();
    }

    fn find_completed_rows(&mut self) {
        'rows: for row in 0..self.level.len() {
            for cell in self.level[row].as_slice() {
                match cell {
                    GameBlock::Filled(_) => {
                        continue;
                    }
                    _ => continue 'rows,
                }
            }
            //If we get here, the row has only filled blocks
            self.completed_rows.push(row);
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
                println!("wtf: {}", self.level[self.level.len()-1][point.x]);
                return true;
            }
            if point.x >= self.level[0].len() {
                println!("wtf!");
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
            PlayerMove::RotateBackward => self.rotate_active_piece(&Rotation::Backward),
            PlayerMove::StepLeft => self.horizontal_move(1, true),
            PlayerMove::StepRight => self.horizontal_move(1, false),
            PlayerMove::OrientDown => self.rotate_active_piece(&Rotation::OrientDown),
            PlayerMove::OrientUp => self.rotate_active_piece(&Rotation::OrientUp),
            PlayerMove::OrientLeft => self.rotate_active_piece(&Rotation::OrientLeft),
            PlayerMove::OrientRight => self.rotate_active_piece(&Rotation::OrientRight),
            _ => false,
        }
    }

    fn vertical_move(&mut self, amount: usize) -> bool {
        if self.active_piece.is_some() {
            let mut point = self.active_piece.as_ref().unwrap().location.clone();
            point.y += amount;
            self.active_piece.as_mut().unwrap().move_to(point);
            if self.ycolliding(self.active_piece.as_ref().unwrap(), None) {
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
            if self.is_xcolliding(self.active_piece.as_ref().unwrap(), None) {
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

            if self.is_xcolliding(self.active_piece.as_ref().unwrap(), None) {
                self.active_piece.as_mut().unwrap().rollback_rotation();
                return false;
            }
            if self.ycolliding(self.active_piece.as_ref().unwrap(), None) {
                self.active_piece.as_mut().unwrap().rollback_rotation();
                return false;
            }
        }
        true
    }

    fn game_over(&mut self) {}

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

    fn send_state_to_ui(&mut self, state: GameState) -> bool {
        if self.ui_slave.is_some() {
            return self.ui_slave.as_ref().unwrap().channel_out.send(Master2UICommunique {
                comm_type: Update,
                state: Some(state),
                piece: None,
            }).is_err();
        }
        return false;
    }

    fn send_render_update(&mut self, state: GameState) -> bool {
        if self.render_slave.is_some() {
            let mut level_update = self.level.clone();
            self.active_piece.as_ref().unwrap().place_in_matrix(level_update.as_mut_slice());
            return self.render_slave.as_ref().unwrap().channel_out.send(Master2RenderCommunique {
                comm_type: Communique::Update,
                level: Some(level_update),
                active_piece: None,
                state: Some(state),
                score: Some(self.score),
            }).is_err();
        }
        return false;
    }
}
