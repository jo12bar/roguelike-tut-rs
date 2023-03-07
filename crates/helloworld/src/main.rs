use rltk::{GameState, Rltk, RltkBuilder, RGB};

struct State {
    y: i32,
    going_down: bool,
}

impl Default for State {
    fn default() -> Self {
        Self {
            y: 1,
            going_down: true,
        }
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        let col_cyan = RGB::named(rltk::CYAN);
        let col_yellow = RGB::named(rltk::YELLOW);
        let percent: f32 = self.y as f32 / 50.0;
        let fg = col_cyan.lerp(col_yellow, percent);

        ctx.cls();
        ctx.print_color(
            1,
            self.y,
            fg,
            RGB::named(rltk::BLACK),
            "♫ ♪ Hello RLTK World ☺",
        );

        if self.going_down {
            self.y += 1;
            if self.y > 48 {
                self.going_down = false;
            }
        } else {
            self.y -= 1;
            if self.y < 2 {
                self.going_down = true;
            }
        }

        ctx.draw_box(
            39,
            0,
            20,
            3,
            RGB::named(rltk::WHITE),
            RGB::named(rltk::BLACK),
        );
        ctx.print_color(
            40,
            1,
            RGB::named(rltk::YELLOW),
            RGB::named(rltk::BLACK),
            &format!("FPS: {}", ctx.fps),
        );
        ctx.print_color(
            40,
            2,
            RGB::named(rltk::CYAN),
            RGB::named(rltk::BLACK),
            &format!("Frame Time: {} ms", ctx.frame_time_ms),
        );
    }
}

fn main() -> rltk::BError {
    let context = RltkBuilder::simple80x50()
        .with_title("Hello RLTK World")
        .with_fps_cap(30.0)
        .build()?;

    let gs = State::default();

    rltk::main_loop(context, gs)
}
