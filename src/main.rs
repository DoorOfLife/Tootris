use std::io::{stdout, Stdout, Write};
use std::sync::mpsc::{channel, Receiver, Sender};

use crossterm::{
    cursor, ExecutableCommand,
    QueueableCommand, Result, style::{self, Colorize}, terminal,
};

use game::game_loop_controller::EvilGameMaster;
use game::tootris::{GameBlock, GameBroadcaster, GameUpdateReceiver, Master2RenderCommunique, UI2MasterCommunique};
use ui::crossterm_render;

use crate::ui::crossterm_render::TermRenderer;
use crate::game::tootris::Renderer;

mod tests;
mod settings;
mod ui;
mod game;

fn main() -> Result<()> {
    let gm_2_render: (Sender<Master2RenderCommunique>, Receiver<Master2RenderCommunique>) = channel();
    let gm_to_render_receiver = GameUpdateReceiver {
        receiver: gm_2_render.1,
    };
    let gm_to_render_sender = GameBroadcaster {
        channel_out: gm_2_render.0,
    };

    let ui_to_gm: (Sender<UI2MasterCommunique>, Receiver<UI2MasterCommunique>) = channel();

    let ui_to_gm_receiver = GameUpdateReceiver {
        receiver: ui_to_gm.1,
    };
    let ui_to_gm_sender = GameBroadcaster {
        channel_out: ui_to_gm.0,
    };

    let mut master = EvilGameMaster::new(50, 50, None,
                                         Some(gm_to_render_sender), None, Some(ui_to_gm_receiver));

    let mut my_renderer = TermRenderer {
        from_master: Some(gm_to_render_receiver),
        from_ui: None,
        out: Some(stdout()),
        current_matrix: Some(master.level.clone()),
        draw_buffer: None,
        term_size: None,
        state: None
    };
    master.new_game();
    master.resume_game();
    my_renderer.full_refresh();
    let mut run = true;
    while run {
        run = master.process_game();
        run = my_renderer.render();
    }
    Ok(())
}