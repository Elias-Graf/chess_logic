//! Perft (*perf*ormance *t*est) counts the possible end positions given a certain depth.
//! It can be used to find bugs, are there are existing validated results to compare against.
//!
//! Read more about perft: https://www.chessprogramming.org/Perft
//! Validated perft results: https://www.chessprogramming.org/Perft_Results

use chess_logic::{fen::Fen, move_generator, Board};

fn perft_generator(board: &Board, depth: usize) -> usize {
    if depth == 0 {
        return 1;
    }

    let mut move_count = 0;
    let moves = move_generator::all_moves(board);

    for mv in moves {
        let mut board = board.clone();
        board.do_move(mv);

        move_count += perft_generator(&board, depth - 1);
    }

    return move_count;
}

#[test]
fn initial_position() {
    let board =
        Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1 ").unwrap();

    assert_eq!(perft_generator(&board, 1), 20);
    assert_eq!(perft_generator(&board, 2), 400);
    assert_eq!(perft_generator(&board, 3), 8_902);
}

#[test]
fn position_2() {
    let board =
        Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - ")
            .unwrap();

    assert_eq!(perft_generator(&board, 1), 48);
    // assert_eq!(perft_generator(&board, 2), 2_039);
}

#[test]
fn position_3() {
    let board = Board::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - ").unwrap();

    // assert_eq!(perft_generator(&board, 1), 14);
}
