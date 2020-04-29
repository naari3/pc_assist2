use game_util::prelude::*;
use game_util::GameloopCommand;
use glutin::*;
use std::sync::mpsc::Receiver;

pub struct Game {
    context: WindowedContext<PossiblyCurrent>,
    lsize: dpi::LogicalSize,
    drift: f64,
    counter: f64,
    start: std::time::Instant,
    text: game_util::TextRenderer,
    sprite_batch: game_util::SpriteBatch,
    sprites: sprites::Sprites,
    recv: Receiver<&'static str>,
}

impl Game {
    pub fn new(
        context: WindowedContext<PossiblyCurrent>,
        lsize: dpi::LogicalSize,
        recv: Receiver<&'static str>,
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
        GameloopCommand::Continue
    }

    fn render(&mut self, alpha: f64, smooth_delta: f64) {
        let dpi = self.context.window().get_hidpi_factor();
        self.text.dpi = dpi as f32;
        self.text.screen_size = (self.lsize.width as f32, self.lsize.height as f32);

        self.text.draw_text(
            &format!(
                "FPS: {:.1}\nDrift: {:.3}\nAlpha: {:.1}\nDPI: {:.1}",
                1.0 / smooth_delta,
                self.drift,
                alpha,
                dpi
            ),
            15.0,
            350.0,
            game_util::Alignment::Left,
            [255; 4],
            32.0,
            0,
        );
        self.text.draw_text(
            concat!(
                "These characters aren't in Noto Sans,\n",
                "but we can still draw them because we have\n",
                "fallback fonts: 你好，世界！\n",
                "(that's \"Hello world!\" in Chinese)"
            ),
            15.0,
            160.0,
            game_util::Alignment::Left,
            [0, 0, 0, 255],
            28.0,
            0,
        );
        self.text.draw_text(
            "16px",
            10.0,
            10.0,
            game_util::Alignment::Left,
            [0, 0, 0, 255],
            16.0,
            0,
        );

        self.sprite_batch
            .draw(&self.sprites.plan[1], point2(20.0, 10.0), [0xFF; 4]);
        self.sprite_batch
            .draw(&self.sprites.plan[3], point2(20.0, 11.0), [0xFF; 4]);
        self.sprite_batch
            .draw(&self.sprites.plan[3], point2(20.0, 12.0), [0xFF; 4]);
        self.sprite_batch
            .draw(&self.sprites.plan[2], point2(20.0, 13.0), [0xFF; 4]);

        let (width, height): (u32, _) = self.lsize.to_physical(dpi).into();
        let (width, height) = (width as i32, height as i32);

        unsafe {
            gl::Viewport(0, 0, width, height);

            gl::ClearColor(0.3, 0.3, 0.9, 0.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        }

        self.text.render();
        self.sprite_batch
            .render(Transform3D::ortho(0.0, 40.0, 0.0, 23.0, -1.0, 1.0));

        self.context.swap_buffers().unwrap();
    }

    fn event(&mut self, event: WindowEvent, _: WindowId) -> GameloopCommand {
        match event {
            WindowEvent::CloseRequested => return GameloopCommand::Exit,
            WindowEvent::Resized(new_size) => {
                let psize = new_size.to_physical(self.context.window().get_hidpi_factor());
                println!("{}, {}", psize.width, psize.height);
                self.context
                    .resize(glutin::dpi::PhysicalSize::new(1280.0, 720.0));
                self.lsize = new_size;
            }
            _ => {}
        }
        GameloopCommand::Continue
    }
}
