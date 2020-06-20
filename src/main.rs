extern crate pcf;
extern crate process_memory;
use crate::plan::{Cells, PlanPlacement};
use board::{Board, BoardEvent};
use ppt::Ppt;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;

#[cfg(windows)]
extern crate winapi;

mod board;
mod plan;
mod ppt;
mod window;

#[cfg(not(windows))]
fn main() {
    println!("pc_assist can only be run on Windows.")
}

/// A helper function to get a Pid from the name of a process
#[cfg(windows)]
pub fn get_pid(process_name: &str) -> process_memory::Pid {
    /// A helper function to turn a c_char array to a String
    fn utf8_to_string(bytes: &[i8]) -> String {
        use std::ffi::CStr;
        unsafe {
            CStr::from_ptr(bytes.as_ptr())
                .to_string_lossy()
                .into_owned()
        }
    }
    let mut entry = winapi::um::tlhelp32::PROCESSENTRY32 {
        dwSize: std::mem::size_of::<winapi::um::tlhelp32::PROCESSENTRY32>() as u32,
        cntUsage: 0,
        th32ProcessID: 0,
        th32DefaultHeapID: 0,
        th32ModuleID: 0,
        cntThreads: 0,
        th32ParentProcessID: 0,
        pcPriClassBase: 0,
        dwFlags: 0,
        szExeFile: [0; winapi::shared::minwindef::MAX_PATH],
    };
    let snapshot: process_memory::ProcessHandle;
    unsafe {
        snapshot = winapi::um::tlhelp32::CreateToolhelp32Snapshot(
            winapi::um::tlhelp32::TH32CS_SNAPPROCESS,
            0,
        );
        if winapi::um::tlhelp32::Process32First(snapshot, &mut entry)
            == winapi::shared::minwindef::TRUE
        {
            while winapi::um::tlhelp32::Process32Next(snapshot, &mut entry)
                == winapi::shared::minwindef::TRUE
            {
                if utf8_to_string(&entry.szExeFile) == process_name {
                    return entry.th32ProcessID;
                }
            }
        }
    }
    0
}

fn run(send: Sender<BoardEvent>, ppt_pid: process_memory::Pid) {
    use process_memory::*;
    use std::collections::HashSet;
    use std::iter::FromIterator;

    let process_handler: ProcessHandle = ppt_pid.try_into_process_handle().unwrap();
    let ppt = Ppt {
        process_handle: process_handler,
    };

    let mut board = Board {
        columns: vec![],
        current_piece: None,
        hold: None,
        next_pieces: vec![],
    };
    let mut mino_set = HashSet::<u32>::new();
    let mut preview_next_pieces: Vec<u32> = vec![];
    let mut _put_count = 0;

    let mut player_index = ppt.find_player_index().unwrap();

    let mut can_prediction = ppt.get_current_piece(player_index).is_none();
    let mut need_reset = can_prediction;
    let mut resetted = !need_reset;

    while ppt.still_active().unwrap() {
        let current_piece = ppt.get_current_piece(player_index);
        if current_piece.is_none() && !need_reset && !resetted {
            can_prediction = true;
            need_reset = true;
        }
        if current_piece.is_none() && need_reset && !resetted {
            let next_pieces = match ppt.get_next_pieces(player_index) {
                Ok(i) => i,
                Err(_e) => continue,
            };
            mino_set = HashSet::<_>::from_iter(
                HashSet::<_>::from_iter(vec![0, 1, 2, 3, 4, 5, 6].into_iter())
                    .difference(&HashSet::<_>::from_iter(next_pieces.clone().into_iter()))
                    .into_iter()
                    .copied(),
            );
            _put_count = 0;
            preview_next_pieces = next_pieces.clone();
            resetted = true;
            need_reset = false;
            player_index = ppt.find_player_index().unwrap();
        }
        if mino_set.is_empty() {
            mino_set = HashSet::<_>::from_iter(vec![0, 1, 2, 3, 4, 5, 6].into_iter());
        }
        if current_piece == board.current_piece {
            continue;
        }

        resetted = false;
        board.current_piece = current_piece;

        board.columns = match ppt.get_columns(player_index) {
            Ok(v) => v,
            Err(_e) => continue,
        };

        let mut next_pieces = ppt.get_next_pieces(player_index).unwrap();
        if can_prediction {
            if next_pieces != preview_next_pieces {
                _put_count += 1;
                if let Some(i) = next_pieces.last() {
                    mino_set.retain(|&j| j != *i);
                }
            }
            preview_next_pieces = preview_next_pieces;

            if mino_set.len() == 1 {
                for i in mino_set.iter() {
                    next_pieces.push(*i);
                }
            }
        }
        board.next_pieces = next_pieces.clone();
        board.hold = ppt.get_hold(player_index).ok();

        send.send(BoardEvent::Continue(board.clone())).ok();
    }

    println!("PPT closed");
    send.send(BoardEvent::Exit).ok();
}

fn run_window(recv: Receiver<Arc<Option<Cells>>>, ppt_pid: process_memory::Pid) {
    use game_util::prelude::*;

    let mut events = glutin::EventsLoop::new();
    let (context, lsize) = game_util::create_context(
        glutin::WindowBuilder::new()
            .with_transparency(true)
            .with_always_on_top(true)
            .with_resizable(true),
        0,
        true,
        &mut events,
    );

    let mut game = window::Game::new(context, lsize, recv, ppt_pid);
    game_util::gameloop(&mut events, &mut game, 60.0, true);
    println!("window closed");
}

include!(concat!(env!("OUT_DIR"), "/sprites.rs"));

#[cfg(windows)]
fn main() -> std::io::Result<()> {
    use std::thread;

    let ppt_pid = get_pid("puyopuyotetris.exe");

    let mut prev_soln: Vec<pcf::Placement> = vec![];

    let (window_send, window_recv) = channel();
    let (board_send, board_recv) = channel();

    thread::spawn(move || run_window(window_recv, ppt_pid));
    thread::spawn(move || run(board_send, ppt_pid));

    let mut count = 0;

    loop {
        match board_recv.recv().unwrap() {
            BoardEvent::Continue(board) => {
                count += 1;
                println!("UPDATE {}", count);
                let s = Arc::new(None);
                window_send.send(Arc::clone(&s)).unwrap();
                pcf::solve_pc(
                    &board.get_queue(),
                    board.get_bitboard(),
                    true,
                    true,
                    pcf::placeability::simple_srs_spins,
                    |soln| {
                        let soln_vec = soln.clone().to_vec();
                        if soln_vec != prev_soln {
                            prev_soln = soln_vec;
                            println!("PC: {:?}", soln);
                            let s = Arc::new(Some(soln[0].cells().clone()));
                            window_send.send(Arc::clone(&s)).unwrap();
                        }
                        pcf::SearchStatus::Abort
                    },
                );
            }
            BoardEvent::Exit => break,
        }
    }

    println!("close");
    Ok(())
}
