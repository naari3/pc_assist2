extern crate process_memory;
use process_memory::{DataMember, Memory, ProcessHandle};

pub struct Ppt {
    pub process_handle: ProcessHandle,
}

impl Ppt {
    fn get_base_address(&self) -> usize {
        0x7FF7_F787_0000 // TODO: elegant fetch base address
    }

    pub fn still_active(&self) -> std::io::Result<bool> {
        let mut exit_code: winapi::shared::minwindef::DWORD = 0;
        if unsafe {
            winapi::um::processthreadsapi::GetExitCodeProcess(self.process_handle, &mut exit_code)
        } == winapi::shared::minwindef::FALSE
        {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(exit_code == winapi::um::minwinbase::STILL_ACTIVE)
        }
    }

    pub fn get_current_piece(&self, index: u32) -> Option<u32> {
        let offsets;
        if index == 0 {
            offsets = vec![self.get_base_address() + 0x01F260D0, 0x1CC0, 0x8];
        } else {
            offsets = todo!();
        }
        let current_piece_address = DataMember::<i32>::new_offset(self.process_handle, offsets);
        let current_piece = current_piece_address.read().ok().and_then(|i| match i {
            0..=6 => Some(i as u32),
            _ => None,
        });
        return current_piece;
    }

    pub fn get_columns(&self, index: u32) -> std::io::Result<Vec<Vec<i32>>> {
        let offsets;
        if index == 0 {
            offsets = vec![self.get_base_address() + 0x01F260D0, 0x1CB8, 0x18];
        } else {
            offsets = todo!();
        }

        let board_address = DataMember::<u64>::new_offset(self.process_handle, offsets);
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

    pub fn get_next_pieces(&self, index: u32) -> std::io::Result<Vec<u32>> {
        let offsets;
        if index == 0 {
            offsets = vec![self.get_base_address() + 0x01F260D0, 0x60, 0x98, 0x168];
        } else {
            offsets = todo!();
        }
        let next_pieces_address = DataMember::<[u64; 5]>::new_offset(self.process_handle, offsets);
        let next_pieces = next_pieces_address
            .read()?
            .to_vec()
            .iter()
            .map(|p| (p & 0x0000FFFF) as u32)
            .collect();

        return Ok(next_pieces);
    }

    pub fn get_hold(&self, index: u32) -> std::io::Result<u32> {
        let offsets;
        if index == 0 {
            offsets = vec![self.get_base_address() + 0x01F260D0, 0x1CC8, 0x8];
        } else {
            offsets = todo!();
        }
        let hold_address = DataMember::<u32>::new_offset(self.process_handle, offsets);
        let hold = hold_address.read()?;

        return Ok(hold);
    }

    pub fn get_player_count(&self) -> std::io::Result<u32> {
        return Ok(1);

        let player_count_address = DataMember::<u32>::new_offset(
            self.process_handle,
            vec![self.get_base_address() + 0x01F260D0, 0x20, 0xB4],
        );

        let player_count_result = player_count_address.read();
        let player_count = match player_count_result {
            Ok(i) => i,
            Err(_) => 0,
        };

        if player_count > 4 {
            return Ok(0);
        }

        // if player_count < 0 {
        //     return Ok(0);
        // }

        return Ok(player_count);
    }

    pub fn get_local_steam(&self) -> std::io::Result<u32> {
        let local_steam_address =
            DataMember::<u32>::new_offset(self.process_handle, vec![0x1405A2010]);
        let local_steam = local_steam_address.read()?;

        return Ok(local_steam);
    }

    pub fn get_player_steam(&self, player: u32) -> std::io::Result<u32> {
        let player_steam_address = DataMember::<u32>::new_offset(
            self.process_handle,
            vec![
                self.get_base_address() + 0x01F260D0,
                0x20,
                (0x118 + player * 0x50) as usize,
            ],
        );
        let player_steam = player_steam_address.read()?;

        return Ok(player_steam);
    }

    pub fn find_player_index(&self) -> std::io::Result<u32> {
        if self.get_player_count()? < 2 {
            return Ok(0);
        }

        let local_steam = self.get_local_steam()?;
        for i in 1..2 {
            if local_steam == self.get_player_steam(i as u32)? {
                return Ok(i as u32);
            }
        }

        return Ok(0);
    }
}
