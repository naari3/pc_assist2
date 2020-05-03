extern crate process_memory;
use process_memory::{DataMember, Memory, ProcessHandle};

pub struct Ppt {
    pub process_handle: ProcessHandle,
}

impl Ppt {
    pub fn still_active(&self) -> bool {
        let mut exit_code: winapi::shared::minwindef::DWORD = 0;
        unsafe {
            winapi::um::processthreadsapi::GetExitCodeProcess(self.process_handle, &mut exit_code)
        };
        exit_code == winapi::um::minwinbase::STILL_ACTIVE
    }

    pub fn get_current_piece(&self) -> Option<u32> {
        let current_piece_address = DataMember::<i32>::new_offset(
            self.process_handle,
            vec![0x140461B20, 0x378, 0x40, 0x140, 0x110],
        );
        let current_piece = current_piece_address.read().ok().and_then(|i| {
            if i == -1 {
                return None;
            } else {
                return Some(i as u32);
            }
        });
        return current_piece;
    }

    pub fn get_columns(&self) -> std::io::Result<Vec<Vec<i32>>> {
        let board_address = DataMember::<u32>::new_offset(
            self.process_handle,
            vec![0x140461B20, 0x378, 0xC0, 0x10, 0x3C0, 0x18],
        );
        let mut columns_addresses = DataMember::<[u64; 10]>::new(self.process_handle);
        columns_addresses.set_offset(vec![board_address.read()? as usize]);
        let column_addrs = columns_addresses.read()?;

        let mut columns: Vec<Vec<i32>> = Vec::new();
        for column_addr in column_addrs.iter() {
            let mut pieces = DataMember::<[i32; 40]>::new(self.process_handle);
            pieces.set_offset(vec![*column_addr as usize]);
            columns.push(pieces.read()?.to_vec());
        }

        return Ok(columns);
    }

    pub fn get_next_pieces(&self) -> std::io::Result<Vec<u32>> {
        let next_pieces_address = DataMember::<[u32; 5]>::new_offset(
            self.process_handle,
            vec![0x140461B20, 0x378, 0xB8, 0x15C],
        );
        let next_pieces = next_pieces_address.read()?.to_vec();

        return Ok(next_pieces);
    }

    pub fn get_hold(&self) -> std::io::Result<u32> {
        let hold_address =
            DataMember::<u32>::new_offset(self.process_handle, vec![0x140598A20, 0x38, 0x3D0, 0x8]);
        let hold = hold_address.read()?;

        return Ok(hold);
    }
}
