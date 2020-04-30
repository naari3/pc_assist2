use crate::plan::Cells;
use game_util::prelude::*;
use game_util::GameloopCommand;
use glutin::*;
use std::sync::mpsc::Receiver;
use std::sync::Arc;

#[cfg(windows)]
extern crate winapi;

pub struct Game {
    context: WindowedContext<PossiblyCurrent>,
    lsize: dpi::LogicalSize,
    drift: f64,
    counter: f64,
    start: std::time::Instant,
    text: game_util::TextRenderer,
    sprite_batch: game_util::SpriteBatch,
    sprites: sprites::Sprites,
    recv: Receiver<Arc<Option<Cells>>>,
    cells: Arc<Option<Cells>>,
}

impl Game {
    pub fn new(
        context: WindowedContext<PossiblyCurrent>,
        lsize: dpi::LogicalSize,
        recv: Receiver<Arc<Option<Cells>>>,
    ) -> Game {
        let (sprites, sprite_sheet) = sprites::Sprites::load();
        Game {
            context,
            lsize,
            drift: 0.0,
            counter: 0.0,
            start: std::time::Instant::now(),
            text: {
                use game_util::rusttype::Font;
                let mut t = game_util::TextRenderer::new();
                t.add_style(ArrayVec::from([Font::from_bytes(include_bytes!(
                    "fonts/NotoSans-Regular.ttf"
                )
                    as &[u8])
                .unwrap()]));
                t
            },
            sprite_batch: {
                let mut sprite_batch =
                    game_util::SpriteBatch::new(game_util::sprite_shader(), sprite_sheet);
                sprite_batch.pixels_per_unit = 83.0;
                sprite_batch
            },
            sprites: sprites,
            recv: recv,
            cells: Arc::new(None),
        }
    }
}

include!(concat!(env!("OUT_DIR"), "/sprites.rs"));

impl game_util::Game for Game {
    fn update(&mut self) -> GameloopCommand {
        let time = std::time::Instant::now() - self.start;
        self.counter += 1.0 / 60.0;
        self.drift = self.counter - time.as_nanos() as f64 / 1_000_000_000.0;
        // self.context
        //     .window()
        //     .set_position(dpi::LogicalPosition::new(100.0, 100.0));
        if let Some(s) = self.recv.try_recv().ok() {
            self.cells = s;
        }
        GameloopCommand::Continue
    }

    fn render(&mut self, alpha: f64, smooth_delta: f64) {
        let dpi = self.context.window().get_hidpi_factor();
        self.text.dpi = dpi as f32;
        self.text.screen_size = (self.lsize.width as f32, self.lsize.height as f32);

        // for y in (0..20).rev() {
        //     for x in (0..10).rev() {
        //         self.sprite_batch.draw(
        //             &self.sprites.plan[0],
        //             point2(x as f32 + 9.20, y as f32 + 6.1),
        //             [255; 4],
        //         );
        //     }
        // }

        if let Some(cells) = *self.cells {
            for &(x, y, d) in &cells {
                self.sprite_batch.draw(
                    &self.sprites.plan[d.to_bits() as usize],
                    point2(x as f32 + 9.20, y as f32 + 6.1),
                    [255; 4],
                );
            }
        }

        let (width, height): (u32, _) = self.lsize.to_physical(dpi).into();
        let (width, height) = (width as i32, height as i32);

        unsafe {
            gl::Viewport(0, 0, width, height);

            gl::ClearColor(0.0, 0.0, 0.0, 0.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        }

        self.text.render();
        self.sprite_batch
            .render(Transform3D::ortho(0.0, 54.0, 0.0, 30.05, -1.0, 1.0));

        self.context.swap_buffers().unwrap();
    }

    fn event(&mut self, event: WindowEvent, _: WindowId) -> GameloopCommand {
        match event {
            WindowEvent::CloseRequested => return GameloopCommand::Exit,
            WindowEvent::Resized(new_size) => {
                println!("{:?}", new_size);
                let psize = new_size.to_physical(self.context.window().get_hidpi_factor());
                self.context.resize(psize);
                self.lsize = new_size;
            }
            _ => {}
        }
        GameloopCommand::Continue
    }
}

use std::mem;
use std::os::raw::c_void;
use winapi::shared::minwindef::*;
use winapi::shared::windef::*;
use winapi::um::winuser::{
    EnumWindows, GetWindow, GetClientRect, GetWindowRect, GetWindowThreadProcessId, IsWindowVisible, GW_OWNER,
};

#[derive(Debug)]
struct HandleData {
    process_id: DWORD,
    window_handle: HWND,
}

pub fn get_client_rect_by_process_id(pid: DWORD) -> RECT {
    let hwnd = get_window_handle_by_process_id(pid);

    let mut rect = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };

    unsafe {
        GetClientRect(hwnd, &mut rect);
    }
    rect
}

pub fn get_window_rect_by_process_id(pid: DWORD) -> RECT {
    let hwnd = get_window_handle_by_process_id(pid);

    let mut rect = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };

    unsafe {
        GetWindowRect(hwnd, &mut rect);
    }
    rect
}
fn get_window_handle_by_process_id(pid: DWORD) -> HWND {
    let handle_data = HandleData {
        process_id: pid,
        window_handle: 0 as HWND,
    };

    unsafe {
        EnumWindows(
            Some(enum_proc),
            (&handle_data as *const HandleData) as LPARAM,
        );
    }

    handle_data.window_handle
}

unsafe extern "system" fn enum_proc(hwnd: HWND, l_param: LPARAM) -> BOOL {
    let handle_data: &mut HandleData = { mem::transmute(l_param as *mut c_void) };
    let mut process_id: DWORD = 0;
    GetWindowThreadProcessId(hwnd, &mut process_id);
    if handle_data.process_id != process_id || !is_main_window(hwnd) {
        return TRUE;
    }
    handle_data.window_handle = hwnd;
    FALSE
}

unsafe fn is_main_window(hwnd: HWND) -> bool {
    return GetWindow(hwnd, GW_OWNER) == 0 as HWND && IsWindowVisible(hwnd) != 0;
}
