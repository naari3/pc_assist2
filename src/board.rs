extern crate pcf;

#[derive(Clone, Debug)]
pub struct Board {
    pub columns: Vec<Vec<i32>>,
    pub current_piece: Option<u32>,
    pub hold: Option<u32>,
    pub next_pieces: Vec<u32>,
}

impl Board {
    pub fn get_queue(&self) -> Vec<pcf::Piece> {
        let mut queue = self.next_pieces.clone();
        self.hold.and_then(|i| Some(queue.insert(0, i)));
        self.current_piece.and_then(|i| Some(queue.insert(0, i)));

        println!("queue: {:?}", queue);
        return queue.into_iter().map(|i| pcf::PIECES[i as usize]).collect();
    }

    pub fn get_bitboard(&self) -> pcf::BitBoard {
        let mut bits: u64 = 0b0;
        for y in (0..20).rev() {
            bits <<= 10;
            let mut row: u64 = 0;
            for x in (0..10).rev() {
                row <<= 1;
                if -1 != self.columns[x][y] {
                    row += 1;
                }
            }
            bits += row;
        }
        return pcf::BitBoard(bits);
    }
}

pub enum BoardEvent {
    Exit,
    Continue(Board),
}
