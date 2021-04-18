mod tootris;
mod piece_types;
mod tests;
mod piece;
mod settings;
mod game_loop_controller;
mod ui;

use ui::crossterm_render;
use std::io::{stdout, Write, Stdout};
use crossterm::{
    ExecutableCommand, QueueableCommand,
    terminal, cursor, style::{self, Colorize}, Result,
};
use crate::ui::crossterm_render::TermRenderer;
use crate::game_loop_controller::EvilGameMaster;
use std::sync::mpsc::{channel, Sender, Receiver};
use crate::tootris::{GameUpdateReceiver, GameBroadcaster, UI2MasterCommunique, Master2RenderCommunique, GameBlock};
use crate::tootris::GameEngineComponent;

fn main() -> Result<()> {
    let gm_2_render: (Sender<Master2RenderCommunique>, Receiver<Master2RenderCommunique>) = channel();
    let gm_to_render_receiver = GameUpdateReceiver {
        receiver: gm_2_render.1,
        broadcaster: GameEngineComponent::EvilGameMaster,
    };
    let gm_to_render_sender = GameBroadcaster {
        channel_out: gm_2_render.0,
        receiver: GameEngineComponent::Renderer,
    };

    let ui_to_gm: (Sender<UI2MasterCommunique>, Receiver<UI2MasterCommunique>) = channel();

    let ui_to_gm_receiver = GameUpdateReceiver {
        receiver: ui_to_gm.1,
        broadcaster: GameEngineComponent::Ui,
    };
    let ui_to_gm_sender = GameBroadcaster {
        channel_out: ui_to_gm.0,
        receiver: GameEngineComponent::EvilGameMaster,
    };

    let mut master = EvilGameMaster::new(50, 50, None,
                                         Some(gm_to_render_sender), None, Some(ui_to_gm_receiver));

    let mut my_renderer = TermRenderer {
        master_receiver: Some(gm_to_render_receiver),
        ui_receiver: None,
        out: Some(stdout()),
        current_matrix: Some(master.level.clone()),
        draw_buffer: None,
        term_size: None
    };
    master.new_game();
    my_renderer.full_refresh();
    loop {
        master.process_game();
        my_renderer.check_maybe_render();
    }
    Ok(())
}