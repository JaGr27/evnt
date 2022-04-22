use std::{env, error::Error};

use evnt::App;

fn main() -> Result<(), Box<dyn Error>> {
    let app = App::new(&format!(
        "{}{}",
        env::var("HOME").unwrap(),
        "/.local/share/evnt"
    ));

    evnt::run(app)?;

    Ok(())
}
