//! Perft (*perf*ormance *t*est) counts the possible end positions given a certain depth.
//! It can be used to find bugs, are there are existing validated results to compare against.
//!
//! Read more about perft: https://www.chessprogramming.org/Perft
//! Validated perft results: https://www.chessprogramming.org/Perft_Results

use chess_logic::{
    board::PieceInstance,
    fen::Fen,
    move_generator::{self},
    Board, Square,
};

fn perft(board: &Board, depth: usize, root: bool) -> usize {
    let mut nodes = 0;

    if depth == 0 {
        return 1;
    }

    let moves = move_generator::all_moves(board);

    for mv in moves {
        let mv_src = mv.src();
        let mv_dst = mv.dst();
        let mv_prom = mv.prom_to();
        let mv_color = mv.piece_color();

        let mut board = board.clone();

        if !board.do_move(mv) {
            continue;
        }

        let cnt = perft(&board, depth - 1, false);
        nodes += cnt;

        if root {
            print!(
                "{}{}",
                format!("{:?}", Square::try_from(mv_src).unwrap()).to_lowercase(),
                format!("{:?}", Square::try_from(mv_dst).unwrap()).to_lowercase(),
            );

            if let Some(prom_to) = mv_prom {
                let ins = PieceInstance::new(mv_color, prom_to);
                print!("{}", ins.get_fen());
            }

            println!(": {}", cnt);
        }
    }

    if root {
        println!("\nNodes searched: {}", nodes);
    }

    return nodes;
}

#[test]
fn initial_position() {
    let board =
        Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1 ").unwrap();

    assert_eq!(perft(&board, 1, true), 20);
    assert_eq!(perft(&board, 2, true), 400);
    assert_eq!(perft(&board, 3, true), 8_902);
}

#[test]
fn position_2() {
    let board =
        Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - ")
            .unwrap();

    assert_eq!(perft(&board, 1, true), 48);
    assert_eq!(perft(&board, 2, true), 2_039);
    assert_eq!(perft(&board, 3, true), 97_862);
}

#[test]
fn position_3() {
    let board = Board::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - ").unwrap();

    assert_eq!(perft(&board, 1, true), 14);
    assert_eq!(perft(&board, 2, true), 191);
    assert_eq!(perft(&board, 3, true), 2812);
}

#[test]
fn position_4() {
    let board = Board::from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1")
        .unwrap();

    assert_eq!(perft(&board, 1, true), 6);
    assert_eq!(perft(&board, 2, true), 264);
    assert_eq!(perft(&board, 3, true), 9_467);
}

#[test]
fn position_5() {
    let board =
        Board::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8").unwrap();

    assert_eq!(perft(&board, 1, true), 44);
    assert_eq!(perft(&board, 2, true), 1_486);
    assert_eq!(perft(&board, 3, true), 62_379);
}

#[test]
fn position_6() {
    let board =
        Board::from_fen("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10")
            .unwrap();

    assert_eq!(perft(&board, 1, true), 46);
    assert_eq!(perft(&board, 2, true), 2_079);
    assert_eq!(perft(&board, 3, true), 89_890);
}
