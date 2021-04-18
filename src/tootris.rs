use std::sync::mpsc::Sender;
use std::sync::mpsc::Receiver;
use std::fmt;
use std::fmt::{Formatter, Display, Pointer};

use crate::piece::{Piece, PieceState};
use crate::tootris::Rotation::OrientDown;

pub(crate) type GameMatrix = Vec<Vec<GameBlock>>;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum BlockColor {
    Blue,
    Magenta,
    Yellow,
    Green,
    Cyan,
    White,
    Undefined,
}

#[derive(Clone, Debug)]
pub enum GameBlock {
    Filled(BlockColor),
    Origin(BlockColor),
    Empty,
    Indestructible,
    None,
}

impl PartialEq for GameBlock {
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

impl Display for GameBlock {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            GameBlock::Indestructible => write!(f, "¤¤"),
            _ => write!(f,"#"),
        }
    }
}

impl GameBlock {
    pub fn get_color(&self) -> Option<&BlockColor> {
        match self {
            GameBlock::Filled(val) => Some(&val),
            _ => None,
        }
    }
    pub fn get_string_visual(&self) -> &str {
        match self {
            GameBlock::Filled(_) => "#",
            GameBlock::Origin(_) => "#",
            GameBlock::Empty => "-",
            GameBlock::Indestructible => "X",
            _ => {" "}
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum Orientation {
    Normal,
    Forward,
    Backwards,
    UpsideDown,
}

impl Display for Orientation {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Orientation::Normal => write!(f, "Normal"),
            Orientation::Forward => write!(f, "Forward"),
            Orientation::UpsideDown => write!(f, "UpsideDown"),
            Orientation::Backwards => write!(f, "Backwards"),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum Rotation {
    Forward,
    Backward,
    OrientLeft,
    OrientRight,
    OrientUp,
    OrientDown,
}

impl Rotation {
    pub fn perform(&self, orientation: &Orientation) -> Orientation {
        match self {
            Rotation::Backward => {
                match orientation {
                    Orientation::Normal => Orientation::Backwards,
                    Orientation::Backwards => Orientation::UpsideDown,
                    Orientation::UpsideDown => Orientation::Forward,
                    Orientation::Forward => Orientation::Normal,
                }
            }
            Rotation::Forward => {
                match orientation {
                    Orientation::Normal => Orientation::Forward,
                    Orientation::Backwards => Orientation::Normal,
                    Orientation::UpsideDown => Orientation::Backwards,
                    Orientation::Forward => Orientation::UpsideDown,
                }
            }
            Rotation::OrientLeft => Orientation::Backwards,
            Rotation::OrientRight => Orientation::Forward,
            Rotation::OrientUp => Orientation::Normal,
            Rotation::OrientDown => Orientation::UpsideDown,
        }
    }
}


#[derive(Copy, Clone, PartialEq, Debug)]
pub enum GameState {
    Pause,
    Paused,
    Reset,
    Playing,
    Start,
    Tootris,
    PieceFreeze,
    End,
}

impl Display for GameState {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            GameState::Pause => write!(f, "Pause"),
            GameState::Paused => write!(f, "Paused"),
            GameState::Reset => write!(f, "Reset"),
            GameState::Playing => write!(f, "Playing"),
            GameState::Start => write!(f, "Start"),
            GameState::Tootris => write!(f, "Tootris"),
            GameState::PieceFreeze => write!(f, "PieceFreeze"),
            GameState::End => write!(f, "End"),
        }
    }
}

#[derive(Clone, Copy)]
pub struct Point {
    pub x: usize,
    pub y: usize,
}

impl Display for Point {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "({},{})", self.x, self.y)
    }
}

impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}


#[derive(Clone, Copy, PartialEq)]
pub enum PlayerMove {
    StepRight,
    StepLeft,
    StepDown,
    RotateForward,
    RotateBackward,
    HoldLeft,
    HoldRight,
    Drop,
    OrientLeft,
    OrientRight,
    OrientUp,
    OrientDown,
}


#[derive(Copy, Clone)]
pub enum /*"""*/GameEngineComponent/*"""*/ {
    Renderer,
    Ui,
    /* WAAHAHAHAHA..! */
    EvilGameMaster,
}

/**
* Wrappers for communication between the components (UI, Renderer, controller..)
*/
#[derive(Clone)]
pub struct GameBroadcaster<T> {
    pub channel_out: Sender<T>,
    pub receiver: GameEngineComponent,
}

pub struct GameUpdateReceiver<T> {
    pub receiver: Receiver<T>,
    pub broadcaster: GameEngineComponent,
}

pub struct Master2RenderCommunique {
    pub comm_type: Communique,
    pub level: Option<GameMatrix>,
    pub active_piece: Option<Piece>,
    pub state: Option<GameState>,
    pub score: Option<usize>,
}

pub struct Master2UICommunique {
    pub comm_type: Communique,
    pub state: Option<GameState>,
    pub piece: Option<Piece>,
}

pub struct UI2MasterCommunique {
    pub comm_type: Communique,
    pub state: Option<GameState>,
    pub player_move: Option<PlayerMove>,
}

pub struct UI2RenderCommunique {}

#[derive(Clone, Debug)]
pub enum Communique {
    Update,
    Info(&'static str),
    Error(&'static str),
}

pub trait Renderer {
    fn render() {}
    fn give_master_receiver(&mut self, receiver: GameUpdateReceiver<Master2RenderCommunique>);
    fn give_ui_receiver(&mut self, receiver: GameUpdateReceiver<UI2RenderCommunique>);
}

pub trait UIHandler {
    fn process_input(&mut self);
    fn give_master_receiver(&mut self, receiver: GameUpdateReceiver<Master2UICommunique>);
    fn give_master_broadcaster(&mut self, broadcaster: GameBroadcaster<UI2MasterCommunique>);
    fn give_render_broadcaster(&mut self, broadcaster: GameBroadcaster<UI2RenderCommunique>);
}
