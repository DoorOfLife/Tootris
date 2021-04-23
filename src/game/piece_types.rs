use crate::game::piece_types::DefinitionBlock::*;
use std::collections::HashMap;
use crate::settings::{PIECE_LINE, PIECE_SQUARE, PIECE_PODIUM, PIECE_L, PIECE_J, PIECE_S,
                      PIECE_Z, GAME_OVER_PIECE, GAME_OVER_TEXT_1, GAME_OVER_TEXT_2, GAME_OVER_TEXT_3,
                      GAME_OVER_TEXT_4, GAME_OVER_TEXT_5, GAME_OVER_TEXT_6, GAME_OVER_TEXT_7};

use std::borrow::Borrow;

#[macro_export]
macro_rules! double_vec {
    ( $( $x:expr ),* ) => {
        {
            let mut outer_vec = Vec::new();
            $(
                for column in 0..$x.len() {
                    let mut inner_vec = Vec::new();
                    for row in 0..$x[column].len() {
                        inner_vec.push($x[column][row].clone());
                    }
                    outer_vec.push(inner_vec);
                }
            )*
            outer_vec
        }
    };
}

#[derive(Copy, Clone, PartialEq)]
pub enum PieceFreezeProperty {
    Normal,
    /**
    * How much slack, in milliseconds
    */
    FreeSpin,
}

#[derive(Copy, PartialEq)]
pub enum DefinitionBlock {
    Origin,
    Filled,
    Blank,
    Text(&'static str),
}

impl Clone for DefinitionBlock {
    fn clone(&self) -> Self {
        match self {
            Origin => Origin,
            Filled => Filled,
            Blank => Blank,
            Text(val) => Text(val),
        }
    }
}

impl DefinitionBlock {
    pub const fn get_string_visual(&self) -> char {
        match self {
            Origin => '#',
            Filled => '#',
            Blank => ' ',
            _ => ' ',
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct PieceDefinition {
    pub def: Vec<Vec<DefinitionBlock>>,
    pub prop: PieceFreezeProperty,
}

impl PieceDefinition {
    pub fn new(def: Vec<Vec<DefinitionBlock>>, prop: PieceFreezeProperty) -> Self {
        PieceDefinition {
            def,
            prop,
        }
    }
}

pub(crate) type PieceMap = HashMap<&'static str, PieceDefinition>;

pub struct PieceDefinitions {
    piece_map: PieceMap,
}

impl PieceDefinitions {
    pub fn new() -> Self {
        let s = PieceDefinitions {
            piece_map: Self::get_piece_map(),
        };
        return s;
    }

    pub fn get_piece_map() -> PieceMap {
        let mut map = PieceMap::new();
        map.insert(PIECE_LINE, PieceDefinition::new(double_vec!(LINE), PieceFreezeProperty::Normal));
        map.insert(PIECE_SQUARE, PieceDefinition::new(double_vec!(SQUARE), PieceFreezeProperty::Normal));
        map.insert(PIECE_PODIUM, PieceDefinition::new(double_vec!(PODIUM), PieceFreezeProperty::FreeSpin));
        map.insert(PIECE_L, PieceDefinition::new(double_vec!(LPIECE), PieceFreezeProperty::Normal));
        map.insert(PIECE_J, PieceDefinition::new(double_vec!(JPIECE), PieceFreezeProperty::Normal));
        map.insert(PIECE_S, PieceDefinition::new(double_vec!(SPIECE), PieceFreezeProperty::Normal));
        map.insert(PIECE_Z, PieceDefinition::new(double_vec!(ZPIECE), PieceFreezeProperty::Normal));
        map.insert(GAME_OVER_PIECE, PieceDefinition::new(double_vec!(GAME_OVER), PieceFreezeProperty::Normal));

        return map;
    }

    pub fn get_piece_def(&self, key: &str) -> &PieceDefinition {
        let piece = self.piece_map.get(key);
        if piece.is_none() {
            panic!("You gotta give me a (valid) piece, bro");
        }
        piece.unwrap().borrow()
    }
}

pub static SQUARE: [[DefinitionBlock; 2]; 2] = [
    [Origin, Origin],
    [Origin, Origin]];

pub static LINE: [[DefinitionBlock; 4]; 1] =
    [[Filled, Origin, Filled, Filled]];


pub static PODIUM: [[DefinitionBlock; 3]; 2] =
    [
        [Blank, Filled, Blank],
        [Filled, Origin, Filled]
    ];

pub static SPIECE: [[DefinitionBlock; 3]; 2] =
    [
        [Blank, Filled, Filled],
        [Filled, Origin, Blank]];

pub static ZPIECE: [[DefinitionBlock; 3]; 2] =
    [
        [Filled, Origin, Blank],
        [Blank, Filled, Filled]];

pub static LPIECE: [[DefinitionBlock; 2]; 3] =
    [
        [Filled, Blank],
        [Origin, Blank],
        [Filled, Filled]
    ];

pub static JPIECE: [[DefinitionBlock; 2]; 3] =
    [
        [Blank, Filled],
        [Blank, Origin],
        [Filled, Filled]
    ];
pub static GAME_OVER: [[DefinitionBlock; 1]; 8] =
    [
        [Text(GAME_OVER_TEXT_1)],
        [Text(GAME_OVER_TEXT_2)],
        [Text(GAME_OVER_TEXT_3)],
        [Text(GAME_OVER_TEXT_4)],
        [Text(GAME_OVER_TEXT_5)],
        [Text(GAME_OVER_TEXT_6)],
        [Text(GAME_OVER_TEXT_7)],
        [Origin],
    ];