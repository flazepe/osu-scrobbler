mod config;
mod logger;
mod scores;
mod scrobbler;

use scrobbler::Scrobbler;

fn main() {
    Scrobbler::new().start()
}

#[macro_export]
macro_rules! exit {
    ($tag:expr, $message:expr) => {{
        $crate::logger::log_error($tag, $message);
        println!("Press enter to exit.");
        let _ = std::io::stdin().read_line(&mut String::new());
        std::process::exit(1);
    }};
}
