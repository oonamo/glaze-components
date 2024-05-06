use daily_note::note_provider::start_daily_note_watch;
mod config;
mod daily_note;
mod macros;

// TODO: This is very specific to my own preferences, this should be made to handle general cases
fn main() {
    std::panic::set_hook(Box::new(|panic_info| {
        println!("panic_occured: {panic_info:?}");
    }));
    let result = start_daily_note_watch();
    if result.is_err() {
        println!("there was an error in daily_note");
    }
}
