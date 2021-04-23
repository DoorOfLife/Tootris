use std::io::{stdout};
use std::sync::mpsc::{channel, Receiver, Sender};

use crossterm::{
    Result,
};

use game::game_loop_controller::EvilGameMaster;
use game::tootris::{GameBroadcaster, GameUpdateReceiver,
                    Master2RenderCommunique, UI2MasterCommunique};


use crate::ui::crossterm_render::TermRenderer;
use crate::game::tootris::{Renderer, Master2UICommunique, UI2RenderCommunique, UIHandler};
use crate::ui::crossterm_ui::TermUI;


mod tests;
mod settings;
mod ui;
mod game;

use std::env;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let height: usize;
    let width: usize;
    match args.len() {
        2 => {
            height = 24;
            width = args[1].parse().unwrap()
        }
        3 => {
            height = args[2].parse().unwrap();
            width = args[1].parse().unwrap();
        }
        _ => {
            height = 24;
            width = 10
        }
    }
    let gm_2_render: (Sender<Master2RenderCommunique>,
                      Receiver<Master2RenderCommunique>) = channel();

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
    let gm_to_ui: (Sender<Master2UICommunique>, Receiver<Master2UICommunique>) = channel();
    let master_to_ui_sender = GameBroadcaster {
        channel_out: gm_to_ui.0,
    };

    let master_to_ui_receiver = GameUpdateReceiver {
        receiver: gm_to_ui.1,
    };

    let ui_to_render: (Sender<UI2RenderCommunique>, Receiver<UI2RenderCommunique>) = channel();

    let ui_to_render_sender = GameBroadcaster {
        channel_out: ui_to_render.0,
    };

    let ui_to_render_receiver = GameUpdateReceiver {
        receiver: ui_to_render.1,
    };

    let mut ui = TermUI::new(Some(ui_to_gm_sender),
                             Some(ui_to_render_sender),
                             Some(master_to_ui_receiver));

    let mut master = EvilGameMaster::new(height, width, None,
                                         Some(gm_to_render_sender),
                                         Some(master_to_ui_sender),
                                         Some(ui_to_gm_receiver));

    let mut my_renderer = TermRenderer {
        from_master: Some(gm_to_render_receiver),
        from_ui: Some(ui_to_render_receiver),
        out: Some(stdout()),
        current_matrix: Some(master.level.clone()),
        draw_buffer: None,
        term_size: None,
        state: None,
        render_offset: None,
        ui_vector: None,
    };
    master.new_game();
    master.resume_game();
    my_renderer.full_refresh();
    use std::thread;

    let handler = thread::spawn(move || {
        let mut run = true;
        while run {
            run = ui.handle_ui();
        }
    });
    

    let mut run = true;
    while run {
        if !run {break;}
        run = master.process_game();
        if !run {break;}
        run = my_renderer.render();
    }
    handler.join();
    Ok(())
}