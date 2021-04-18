use crate::tootris::{UIHandler, UI2MasterCommunique, GameBroadcaster, GameUpdateReceiver,
                     Master2UICommunique, UI2RenderCommunique, PlayerMove, Communique};
use std::fmt::Debug;
use std::str;
use crate::ui::settings::{MOVE_LEFT_COMMAND, MOVE_RIGHT_COMMAND, ROT_DOWN_COMMAND, ROT_UP_COMMAND,
                          ROT_LEFT_COMMAND, ROT_RIGHT_COMMAND};