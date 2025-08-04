use crate::{history::History, utils, OutputArgs, Result};
use clap::Parser;
use std::fmt::Write;
use std::path::Path;

#[derive(Parser, Debug)]
#[command(about = "Shows the history of all saved snapshots.")]
pub struct LogArgs {
    #[command(flatten)]
    pub output_args: OutputArgs,
}

pub fn run(args: LogArgs) -> Result<()> {
    let root_path = Path::new(".");
    let history = History::load(root_path)?;
    let mut output = String::new();

    if history.snapshots.is_empty() {
        writeln!(&mut output, "No snapshots found. Run `devcat save <message>` to create one.")?;
    } else {
        writeln!(&mut output, "{:<3} {:<22} MESSAGE", "ID", "TIMESTAMP")?;
        writeln!(&mut output, "{:-<3} {:-<22} {:-<50}", "", "", "")?;

        for snapshot in history.snapshots.iter().rev() {
            let ts = snapshot.timestamp.format("%Y-%m-%d %H:%M:%S");
            writeln!(
                &mut output,
                "{:<3} {:<22} {}",
                snapshot.id,
                ts,
                snapshot.message.chars().take(50).collect::<String>()
            )?;
        }
    }

    utils::handle_output(output, &args.output_args, "Log")
}
