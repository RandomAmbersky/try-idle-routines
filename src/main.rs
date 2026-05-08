mod app;
mod core;
mod input;
mod tui;
mod ui;

fn main() -> std::io::Result<()> {
    app::App::new().run()
}
