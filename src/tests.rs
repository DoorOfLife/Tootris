#[cfg(test)]
mod tests {
    use crate::game::tootris::{Rotation, BlockColor, GameBlock, Point, GameState, PlayerMove, UIHandler,
                               GameBroadcaster, GameUpdateReceiver, Master2UICommunique, Renderer,
                               Master2RenderCommunique, UI2RenderCommunique,
                               UI2MasterCommunique, Communique};

    use crate::game::piece_types::*;
    use crate::game::piece::Piece;
    use crate::ui::*;
    use crate::settings::{PIECE_L, PIECE_SQUARE, PIECE_PODIUM, OPTION_TICK_BASE_MS, PIECE_Z, PIECE_S, PIECE_LINE, PIECE_J};
    use std::borrow::Borrow;
    use crate::game::game_loop_controller::EvilGameMaster;
    use std::sync::mpsc::channel;
    use crate::ui::simple_command_ui::SimpleCommandUi;

    #[test]
    fn test_piece() {
        let pieces: PieceDefinitions = PieceDefinitions::new();
        let mut my_piece: Piece = Piece::of_type(pieces.get_piece_def(PIECE_PODIUM),
                                                 BlockColor::Blue, Point { x: 5, y: 8 });
        let mut my_level_owned: Vec<Vec<GameBlock>> = vec![vec![GameBlock::Empty; 10]; 24];
        let mut my_level = my_level_owned.as_mut_slice();
        println!("{}", my_piece);
        my_level = my_piece.place_in_matrix(my_level);
        my_level = my_piece.remove_from_matrix(my_level);
        my_piece.rotate(&Rotation::Forward);
        println!("{}", my_piece);
        my_level = my_piece.place_in_matrix(my_level);
        my_level = my_piece.remove_from_matrix(my_level);
        my_piece.rotate(&Rotation::Forward);
        println!("{}", my_piece);
        my_level = my_piece.place_in_matrix(my_level);
        my_level = my_piece.remove_from_matrix(my_level);
        my_piece.rotate(&Rotation::Forward);
        println!("{}", my_piece);
        my_level = my_piece.place_in_matrix(my_level);
        my_level = my_piece.remove_from_matrix(my_level);
        my_piece.rotate(&Rotation::Forward);
        println!("{}", my_piece);
        my_level = my_piece.place_in_matrix(my_level);
        my_level = my_piece.remove_from_matrix(my_level);
        for y in 0..my_level.len() {
            for x in 0..my_level[0].len() {
                assert_eq!(my_level[y][x], GameBlock::Empty);
            }
        }
    }


    #[test]
    fn test_rotation_tootris_ending() {
        let mut ui = SimpleCommandUi {
            master_receiver: None,
            master_broadcaster: None,
            command_queue: Vec::new(),
        };

        let pieces: PieceDefinitions = PieceDefinitions::new();

        let my_point = Point { x: 5, y: 8 };
        let mut my_piece: Piece = Piece::of_type(pieces.get_piece_def(PIECE_Z),
                                                 BlockColor::Blue, my_point.clone());

        let mut master = EvilGameMaster::new(22, 10, Some(my_piece),
                                             None, None, None);
        let chan_master_render = channel();
        let chan_ui_master = channel();
        ui.give_master_broadcaster(GameBroadcaster {
            channel_out: chan_ui_master.0,
        });
        master.give_ui_receiver(GameUpdateReceiver {
            receiver: chan_ui_master.1,
        });
        master.give_render_slave(GameBroadcaster {
            channel_out: chan_master_render.0,
        });

        let mut mock_renderer = MockCommReceiver::new();
        mock_renderer.give_master_receiver(GameUpdateReceiver
        { receiver: chan_master_render.1 });

        master.resume_game();
        while master.active_piece.as_ref().unwrap().location.y < 10 {
            master.process_game();
            mock_renderer.print_any_update();
            ui.handle_ui();
            ui.submit_command("L");
        }
    }

    #[test]
    fn test_collision_artificial() {
        let pieces: PieceDefinitions = PieceDefinitions::new();

        let my_point = Point { x: 4, y: 2 };
        let other_point = Point { x: 2, y: 4 };

        let mut my_piece: Piece = Piece::of_type(pieces.get_piece_def(PIECE_J), BlockColor::Blue, my_point.clone());
        my_piece.rotate(&Rotation::OrientLeft);
        let mut other_piece: Piece = Piece::of_type(pieces.get_piece_def(PIECE_LINE), BlockColor::Magenta, other_point.clone());
        println!("{}", &my_piece);
        println!("{}", &other_piece);
        let mut master = EvilGameMaster::new(6, 8, Some(my_piece.clone()), None, None, None);
        other_piece.place_in_matrix(master.level.as_mut());

        master.resume_game();
        while master.active_piece.is_some() {
            master.process_game();
            if master.active_piece.is_none() {
                break;
            }
            assert!(master.active_piece.as_ref().unwrap().location.y < 4);
        }

        let matrix = &mut master.level;
        //--DDD-----
        //####D------
        //--------
        assert_eq!(matrix[4][1], GameBlock::Filled(BlockColor::Magenta));
        assert_eq!(matrix[4][2], GameBlock::Filled(BlockColor::Magenta));
        assert_eq!(matrix[4][3], GameBlock::Filled(BlockColor::Magenta));
        assert_eq!(matrix[4][4], GameBlock::Filled(BlockColor::Magenta));
        assert_eq!(matrix[3][3], GameBlock::Filled(BlockColor::Blue));
        assert_eq!(matrix[3][4], GameBlock::Filled(BlockColor::Blue));
        assert_eq!(matrix[3][5], GameBlock::Filled(BlockColor::Blue));
        assert_eq!(matrix[4][5], GameBlock::Filled(BlockColor::Blue));

        master.process_game();

        assert!(master.active_piece.is_some());
        my_piece.location.x += 1;
        master.active_piece = Some(my_piece.clone());

        while master.active_piece.is_some() {
            master.process_game();
            if master.active_piece.is_none() {
                break;
            }
            assert!(master.active_piece.as_ref().unwrap().location.y < 3);
        }
        //---GGG
        //--DDDG----
        //####D------
        //--------
        let matrix = &mut master.level;
        assert_eq!(matrix[4][1], GameBlock::Filled(BlockColor::Magenta));
        assert_eq!(matrix[4][2], GameBlock::Filled(BlockColor::Magenta));
        assert_eq!(matrix[4][3], GameBlock::Filled(BlockColor::Magenta));
        assert_eq!(matrix[4][4], GameBlock::Filled(BlockColor::Magenta));
        assert_eq!(matrix[3][3], GameBlock::Filled(BlockColor::Blue));
        assert_eq!(matrix[3][4], GameBlock::Filled(BlockColor::Blue));
        assert_eq!(matrix[3][5], GameBlock::Filled(BlockColor::Blue));
        assert_eq!(matrix[4][5], GameBlock::Filled(BlockColor::Blue));
        assert_eq!(matrix[2][4], GameBlock::Filled(BlockColor::Blue));
        assert_eq!(matrix[2][5], GameBlock::Filled(BlockColor::Blue));
        assert_eq!(matrix[2][6], GameBlock::Filled(BlockColor::Blue));
        assert_eq!(matrix[3][6], GameBlock::Filled(BlockColor::Blue));

        for y in 0..matrix.len() {
            for x in 0..matrix[0].len() {
                if matrix[y][x] == GameBlock::Indestructible {
                    print!("B");
                } else if matrix[y][x] != GameBlock::Empty{
                    print!("#");
                } else {
                    print!("-");
                }
            }
            println!();
        }
    }


    #[test]
    fn test_game_master_init_ticks_movement_comm() {
        let pieces: PieceDefinitions = PieceDefinitions::new();

        let my_point = Point { x: 5, y: 8 };
        let mut my_piece: Piece = Piece::of_type(pieces.get_piece_def(PIECE_PODIUM), BlockColor::Blue, my_point.clone());
        let mut master = EvilGameMaster::new(22, 10, Some(my_piece), None, None, None);

        let ylen = master.level.len();
        let xlen = master.level[0].len();
        //just checking that the boundaries seem to be present
        assert_eq!(master.level[0][0], GameBlock::Indestructible);
        assert_eq!(master.level[1][0], GameBlock::Indestructible);
        assert_eq!(master.level[2][0], GameBlock::Indestructible);
        assert_eq!(master.level[3][0], GameBlock::Indestructible);
        assert_eq!(master.level[4][0], GameBlock::Indestructible);
        assert_eq!(master.level[5][0], GameBlock::Indestructible);
        assert_eq!(master.level[6][0], GameBlock::Indestructible);
        assert_eq!(master.level[7][0], GameBlock::Indestructible);
        assert_eq!(master.level[8][0], GameBlock::Indestructible);
        assert_eq!(master.level[9][0], GameBlock::Indestructible);
        assert_eq!(master.level[10][0], GameBlock::Indestructible);
        assert_eq!(master.level[11][0], GameBlock::Indestructible);
        assert_eq!(master.level[12][0], GameBlock::Indestructible);
        assert_eq!(master.level[13][0], GameBlock::Indestructible);
        assert_eq!(master.level[14][0], GameBlock::Indestructible);
        assert_eq!(master.level[15][0], GameBlock::Indestructible);
        assert_eq!(master.level[16][0], GameBlock::Indestructible);
        assert_eq!(master.level[17][0], GameBlock::Indestructible);
        assert_eq!(master.level[18][0], GameBlock::Indestructible);
        assert_eq!(master.level[19][0], GameBlock::Indestructible);
        assert_eq!(master.level[20][0], GameBlock::Indestructible);
        assert_eq!(master.level[21][0], GameBlock::Indestructible);
        assert_eq!(master.level[ylen - 1][0], GameBlock::Indestructible);
        assert_eq!(master.level[ylen - 1][1], GameBlock::Indestructible);
        assert_eq!(master.level[ylen - 1][2], GameBlock::Indestructible);
        assert_eq!(master.level[ylen - 1][3], GameBlock::Indestructible);
        assert_eq!(master.level[ylen - 1][4], GameBlock::Indestructible);
        assert_eq!(master.level[ylen - 1][5], GameBlock::Indestructible);
        assert_eq!(master.level[ylen - 1][6], GameBlock::Indestructible);
        assert_eq!(master.level[ylen - 1][7], GameBlock::Indestructible);
        assert_eq!(master.level[ylen - 1][8], GameBlock::Indestructible);
        assert_eq!(master.level[ylen - 1][9], GameBlock::Indestructible);

        let mut bitch: MockCommReceiver = MockCommReceiver::new();
        let chan_master_render = channel();
        let chan_ui_master = channel();

        let master_2_render_receiver: GameUpdateReceiver<Master2RenderCommunique> = GameUpdateReceiver {
            receiver: chan_master_render.1,
        };
        bitch.give_master_receiver(master_2_render_receiver);

        let master_2_render_sender: GameBroadcaster<Master2RenderCommunique> = GameBroadcaster {
            channel_out: chan_master_render.0,
        };
        master.give_render_slave(master_2_render_sender);
        let ui_2_master_sender: GameBroadcaster<UI2MasterCommunique> = GameBroadcaster {
            channel_out: chan_ui_master.0,
        };
        let ui_2_master_receiver: GameUpdateReceiver<UI2MasterCommunique> = GameUpdateReceiver {
            receiver: chan_ui_master.1,
        };
        master.give_ui_receiver(ui_2_master_receiver);
        master.resume_game();
        assert_eq!(master.state, GameState::Playing);
        master.process_game();
        assert_eq!(master.state, GameState::Playing);
        while master.active_piece.as_ref().unwrap().location.y < 9 {
            let move_command = UI2MasterCommunique {
                comm_type: Communique::Update,
                command: None,
                player_move: Some(PlayerMove::StepLeft),
            };
            ui_2_master_sender.channel_out.send(move_command);
            master.process_game();
            bitch.print_any_update();
            assert!(master.sw.is_running());
        }
        println!("Score: {}", master.score);
        assert_eq!(master.active_piece.as_ref().unwrap().location.y, 9);
        //This confirms that when a piece hits the bottom and freezes, a new one spawns somewhere higher up
        while master.active_piece.is_none() || master.active_piece.as_ref().unwrap().location.y > 8 {
            master.process_game();
            bitch.print_any_update();
            bitch.print_any_update();
            assert!(master.sw.is_running());
        }
    }

    pub struct MockCommReceiver {
        master_to_render_receiver: Option<GameUpdateReceiver<Master2RenderCommunique>>,
        ui_to_render_receiver: Option<GameUpdateReceiver<UI2RenderCommunique>>,
    }

    impl MockCommReceiver {
        fn print_any_update(&mut self) {
            if self.master_to_render_receiver.is_some() {
                let result = self.master_to_render_receiver.as_mut().unwrap().receiver.try_recv();
                if result.is_ok() {
                    let update = result.unwrap();
                    if update.state.is_some() {
                        print!("[{}] ", update.state.unwrap());
                    }
                    println!();
                }
            }
        }

        fn new() -> Self {
            MockCommReceiver {
                master_to_render_receiver: None,
                ui_to_render_receiver: None,
            }
        }
    }

    impl Renderer for MockCommReceiver {
        fn render(&mut self) -> bool {
            true
        }
        fn give_master_receiver(&mut self, receiver: GameUpdateReceiver<Master2RenderCommunique>) {
            self.master_to_render_receiver = Some(receiver);
        }

        fn give_ui_receiver(&mut self, receiver: GameUpdateReceiver<UI2RenderCommunique>) {
            todo!()
        }
    }
}

