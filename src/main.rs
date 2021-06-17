use ggez::{graphics, ContextBuilder, GameResult};
use ggez::event;

mod lib;

fn main() -> GameResult<()> {
    let (mut context, event_loop) = ContextBuilder::new("gold_miner", "Amirhosein_GPR").build().expect("Error creating contxet by context builder");

    let game_state = lib::GoldMiner::new(&mut context);

    graphics::set_resizable(&mut context, true)?;
    graphics::set_drawable_size(&mut context, 1280.0, 720.0)?;
    let rect = graphics::Rect::new(0.0, 0.0, 1280.0, 720.0);
    graphics::set_screen_coordinates(&mut context, rect).expect("Couldn't set screen coordinate");

    graphics::set_window_title(&context, "Gold miner");

    event::run(context, event_loop, game_state);
}