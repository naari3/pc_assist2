extern crate pcf;
extern crate process_memory;
#[cfg(windows)]
extern crate winapi;

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

fn create_bit_board_from(columns: Vec<Vec<i32>>) -> pcf::BitBoard {
    let mut bits: u64 = 0b0;
    for y in (0..20).rev() {
        bits <<= 10;
        let mut row: u64 = 0;
        for x in (0..10).rev() {
            row <<= 1;
            if -1 != columns[x][y] {
                row += 1;
            }
        }
        bits += row;
    }
    return pcf::BitBoard(bits);
}

#[cfg(windows)]
fn main() -> std::io::Result<()> {
    use process_memory::*;
    let process_handle = get_pid("puyopuyotetris.exe").try_into_process_handle()?;

    let mut board_address = DataMember::<u32>::new(process_handle);
    board_address.set_offset(vec![0x140461B20, 0x378, 0xC0, 0x10, 0x3C0, 0x18]);

    println!("board address: {}", board_address.read()?);

    let mut columns_addresses = DataMember::<[u64; 10]>::new(process_handle);
    columns_addresses.set_offset(vec![board_address.read()? as usize]);

    let column_addrs = columns_addresses.read()?;
    println!("column addresses: {:?}", column_addrs);

    let mut columns: Vec<Vec<i32>> = Vec::new();

    for (i, column_addr) in column_addrs.iter().enumerate() {
        let mut pieces = DataMember::<[i32; 40]>::new(process_handle);
        pieces.set_offset(vec![*column_addr as usize]);
        columns.push(pieces.read()?.to_vec());
        println!("columns<{}: {}>: {:?}", i, column_addr, &pieces.read()?[..]);
    }
    println!("{:?}", columns);

    let mut queue_address = DataMember::<[u32; 5]>::new(process_handle);
    queue_address.set_offset(vec![0x140461B20, 0x378, 0xB8, 0x15C]);
    let mut queue = queue_address.read()?.to_vec();

    let mut current_piece_address = DataMember::<i32>::new(process_handle);
    current_piece_address.set_offset(vec![0x140461B20, 0x378, 0x40, 0x140, 0x110]);
    let current_piece = current_piece_address.read()?;

    let mut hold_address = DataMember::<u32>::new(process_handle);
    hold_address.set_offset(vec![0x140598A20, 0x38, 0x3D0, 0x8]);
    let hold = match hold_address.read() {
        Ok(i) => Some(i),
        Err(_err) => None,
    };

    println!("queue: {:?}", queue);
    match hold {
        Some(h) => {
            println!("hold: {:?}", h);
            queue.insert(0, h);
        }
        None => {}
    }
    println!("current_piece: {:?}", current_piece);

    if current_piece != -1 {
        queue.insert(0, current_piece as u32);
    }

    println!("{:?}", queue);

    let aa: Vec<pcf::Piece> = queue.into_iter().map(|i| pcf::PIECES[i as usize]).collect();
    for soln in pcf::solve_pc(
        &aa,
        create_bit_board_from(columns),
        true,
        true,
        pcf::placeability::always,
    ) {
        println!("a: {:?}", soln);
    }

    Ok(())
}
