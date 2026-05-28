mod config;
#[cfg(target_os = "macos")]
mod macos;

fn main() {
    #[cfg(target_os = "macos")]
    macos::app::run();

    #[cfg(not(target_os = "macos"))]
    {
        if let Err(error) = config::load_default() {
            eprintln!("Shuttle config error: {error}");
        }
    }
}
