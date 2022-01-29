mod app;

fn main() {
    match app::utils::startup() {
        Ok(stuff) => app::start(stuff),
        Err(e) => {
            eprintln!("error: {}", e);
            std::process::exit(1)
        }
    }
}
