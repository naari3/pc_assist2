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

#[cfg(windows)]
fn main() -> std::io::Result<()> {
    use process_memory::*;
    let process_handle = get_pid("puyopuyotetris.exe").try_into_process_handle()?;
    println!("sss");

    let mut board_address = DataMember::<u32>::new(process_handle);
    board_address.set_offset(vec![0x140461B20, 0x378, 0xC0, 0x10, 0x3C0, 0x18]);

    println!("board address: {}", board_address.read()?);

    let mut columns_addresses = DataMember::<[u64; 10]>::new(process_handle);
    columns_addresses.set_offset(vec![board_address.read()? as usize]);

    let column_addrs = columns_addresses.read()?;
    println!("column addresses: {:?}", column_addrs);

    for (i, column_addr) in column_addrs.iter().enumerate() {
        let mut pieces = DataMember::<[i32; 40]>::new(process_handle);
        pieces.set_offset(vec![*column_addr as usize]);
        if [0, 1, 2, 7, 8, 9].contains(&i) {
            pieces.write(&[0; 40])?;
        } else {
            pieces.write(&[-1; 40])?;
        }
        println!("columns<{}: {}>: {:?}", i, column_addr, &pieces.read()?[..]);
    }

    Ok(())
}
