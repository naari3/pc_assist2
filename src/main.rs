extern crate pcf;
extern crate process_memory;
use board::Board;
use ppt::Ppt;
use std::sync::mpsc::{channel, Sender};

#[cfg(windows)]
extern crate winapi;

mod board;
mod ppt;

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

fn run(send: Sender<Board>) {
    use process_memory::*;
    use std::collections::HashSet;

    let process_handler: ProcessHandle = get_pid("puyopuyotetris.exe")
        .try_into_process_handle()
        .unwrap();
    let ppt = Ppt {
        process_handle: process_handler,
    };

    let mut board = Board {
        columns: vec![],
        current_piece: None,
        hold: None,
        next_pieces: vec![],
    };
    let mut mino_set: HashSet<u32> = vec![].into_iter().collect();
    let mut preview_next_pieces: Vec<u32> = vec![];
    let mut put_count = 0;

    let mut can_prediction = ppt.get_current_piece().is_none();
    let mut need_reset = can_prediction;
    let mut resetted = !need_reset;

    loop {
        let current_piece = ppt.get_current_piece();
        if current_piece.is_none() && !need_reset && !resetted {
            can_prediction = true;
            need_reset = true;
        }
        if current_piece.is_none() && need_reset && !resetted {
            let next_pieces = match ppt.get_next_pieces() {
                Ok(i) => i,
                Err(_e) => continue,
            };
            mino_set = vec![0, 1, 2, 3, 4, 5, 6]
                .into_iter()
                .collect::<HashSet<_>>()
                .difference(&next_pieces.clone().into_iter().collect())
                .into_iter()
                .copied()
                .collect();
            put_count = 0;
            preview_next_pieces = next_pieces.clone();
            resetted = true;
            need_reset = false;
        }
        if mino_set.is_empty() {
            mino_set = vec![0, 1, 2, 3, 4, 5, 6].into_iter().collect();
        }
        if current_piece == board.current_piece {
            continue;
        }

        resetted = false;
        board.current_piece = current_piece;

        board.columns = match ppt.get_columns() {
            Ok(v) => v,
            Err(_e) => continue,
        };

        let mut next_pieces = ppt.get_next_pieces().unwrap();
        if can_prediction {
            if next_pieces != preview_next_pieces {
                put_count += 1;
                match next_pieces.last() {
                    Some(i) => {
                        mino_set.retain(|&j| j != *i);
                    }
                    None => {}
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

        board.hold = ppt.get_hold().ok();

        send.send(board.clone()).ok();

        println!("");
    }
}

#[cfg(windows)]
fn main() -> std::io::Result<()> {
    use std::thread;

    let mut solns: Vec<Vec<pcf::Placement>>;
    let mut prev_solns: Vec<Vec<pcf::Placement>> = vec![];

    let (send, recv) = channel();

    thread::spawn(move || run(send));

    loop {
        let board = recv.recv().unwrap();
        solns = pcf::solve_pc(
            &board.get_queue(),
            board.get_bitboard(),
            true,
            true,
            pcf::placeability::always,
        );
        if solns != prev_solns {
            println!("changed!");
            prev_solns = solns.clone();
            if prev_solns.is_empty() {
                println!("Nothing!")
            } else {
                for soln in solns {
                    println!("PC: {:?}", soln);
                }
            }
            println!("");
        }
    }

    Ok(())
}
