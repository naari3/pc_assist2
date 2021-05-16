use std::io::Result;

use winapi::um::winnt::HANDLE;

use crate::ppt::Ppt;

#[derive(Clone, Debug, PartialEq)]
pub struct State {
    pub columns: Vec<Vec<i32>>,
    pub current_piece: Option<u32>,
    pub hold: Option<u32>,
    pub next_queue: Vec<u32>,
}

impl State {
    pub fn new_from_proc(handle: HANDLE) -> Result<Self> {
        let mut ppt = Ppt {
            process_handle: handle,
        };
        let index = ppt.find_player_index()?;
        Ok(Self {
            columns: ppt.get_columns(index)?,
            current_piece: ppt.get_current_piece(index),
            hold: ppt.get_hold(index)?,
            next_queue: ppt.get_next_pieces(index)?,
        })
    }
}
