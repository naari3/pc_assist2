extern crate pcf;
extern crate process_memory;
use board::Board;
use std::sync::mpsc::{channel, Sender};

#[cfg(windows)]
extern crate winapi;

mod board;

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

    let mut board = Board {
        columns: vec![],
        current_piece: None,
        hold: None,
        next_pieces: vec![],
    };

    let process_handler: ProcessHandle = get_pid("puyopuyotetris.exe").try_into_process_handle().unwrap();

    loop {
        let current_piece_address = DataMember::<i32>::new_offset(
            process_handler,
            vec![0x140461B20, 0x378, 0x40, 0x140, 0x110],
        );
        let current_piece = current_piece_address.read().ok().and_then(|i| {
            if i == -1 {
                return None;
            } else {
                return Some(i as u32);
            }
        });

        if current_piece == board.current_piece {
            continue;
        }

        board.current_piece = current_piece;
        let board_address = DataMember::<u32>::new_offset(
            process_handler,
            vec![0x140461B20, 0x378, 0xC0, 0x10, 0x3C0, 0x18],
        );
        let mut columns_addresses = DataMember::<[u64; 10]>::new(process_handler);
        columns_addresses.set_offset(vec![match board_address.read() {
            Ok(v) => v,
            Err(_e) => continue,
        } as usize]);
        let column_addrs = match columns_addresses.read() {
            Ok(v) => v,
            Err(_e) => continue,
        };

        let mut columns: Vec<Vec<i32>> = Vec::new();
        for column_addr in column_addrs.iter() {
            let mut pieces = DataMember::<[i32; 40]>::new(process_handler);
            pieces.set_offset(vec![*column_addr as usize]);
            columns.push(match pieces.read() {
                Ok(v) => v.to_vec(),
                Err(e) => panic!(e),
            });
        }
        board.columns = columns.clone();

        let queue_address = DataMember::<[u32; 5]>::new_offset(
            process_handler,
            vec![0x140461B20, 0x378, 0xB8, 0x15C],
        );
        let queue = match queue_address.read() {
            Ok(v) => v,
            Err(_e) => continue,
        }
        .to_vec();
        board.next_pieces = queue.clone();

        let hold_address =
            DataMember::<u32>::new_offset(process_handler, vec![0x140598A20, 0x38, 0x3D0, 0x8]);
        let hold = hold_address.read().ok();
        board.hold = hold;

        send.send(board.clone()).ok();
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
