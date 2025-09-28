use colored::ColoredString;
use log::error;

fn gup_main() -> Result<(), ColoredString> {
    Ok(())
}

fn main() {
    human_panic::setup_panic!();
    match gup_main() {
        Ok(()) => (),
        Err(e) => {
            error!("{e}");
            std::process::exit(1);
        }
    }
}
