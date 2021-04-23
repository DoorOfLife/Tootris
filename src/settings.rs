use crate::game::tootris::Point;

pub(crate) static OPTION_TICK_BASE_MS: usize = 4;
//todo: tweak
pub(crate) static OPTION_BUCKET_MAX_SIZE: usize = 20;
pub(crate) static OPTION_BUCKET_MINIMUM_SIZE: usize = 3;
pub(crate) static XRENDER_OFFSET: usize = 20;
pub(crate) static UI_ANCHOR: Point = Point { x: 0, y: 0 };

pub(crate) static PIECE_LINE: &str = "line";
pub(crate) static PIECE_SQUARE: &str = "square";
pub(crate) static PIECE_PODIUM: &str = "podium";
pub(crate) static PIECE_L: &str = "lpiece";
pub(crate) static PIECE_J: &str = "jpiece";
pub(crate) static PIECE_S: &str = "spiece";
pub(crate) static PIECE_Z: &str = "zpiece";
pub(crate) static GAME_OVER_PIECE: &str = "gameoverpiece";

pub(crate) static GAME_OVER_TEXT_1: &str = "██╗---██╗-██████╗-██╗---██╗----███████╗██╗---██╗-██████╗██╗--██╗██╗";
pub(crate) static GAME_OVER_TEXT_2: &str = "╚██╗-██╔╝██╔═══██╗██║---██║----██╔════╝██║---██║██╔════╝██║-██╔╝██║";
pub(crate) static GAME_OVER_TEXT_3: &str = "-╚████╔╝-██║---██║██║---██║----███████╗██║---██║██║-----█████╔╝-██║";
pub(crate) static GAME_OVER_TEXT_4: &str = "--╚██╔╝--██║---██║██║---██║----╚════██║██║---██║██║-----██╔═██╗-╚═╝";
pub(crate) static GAME_OVER_TEXT_5: &str = "---██║---╚██████╔╝╚██████╔╝----███████║╚██████╔╝╚██████╗██║--██╗██╗";
pub(crate) static GAME_OVER_TEXT_6: &str = "---╚═╝----╚═════╝--╚═════╝-----╚══════╝-╚═════╝--╚═════╝╚═╝--╚═╝╚═╝";
pub(crate) static GAME_OVER_TEXT_7: &str = "-------------------------------------------------------------------";