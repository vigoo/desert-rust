use std::io::{self, Read};

fn main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    if matches!(args.next().as_deref(), Some("supports")) {
        std::process::exit(0);
    }

    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let output = desert_book::preprocess_json(&input)?;
    println!("{output}");
    Ok(())
}
