use crate::{history::History, utils, Result, OutputArgs};
use clap::Parser;
use std::fmt::Write;
use std::path::Path;

#[derive(Parser, Debug)]
#[command(about = "Shows a list of all files in a specific snapshot.")]
pub struct InspectArgs {
    /// The snapshot ID to inspect.
    #[arg(required = true)]
    pub id: u32,
    #[command(flatten)]
    pub output_args: OutputArgs,
}

pub fn run(args: InspectArgs) -> Result<()> {
    let root_path = Path::new(".");
    let history = History::load(root_path)?;
    let snapshot = history.get_snapshot(args.id)?;

    let manifest = utils::get_manifest_from_hash(root_path, &snapshot.manifest_hash)?;

    let mut output = String::new();
    writeln!(&mut output, "Files in snapshot {} ({}):", snapshot.id, snapshot.message)?;
    for path in manifest.keys() {
        writeln!(&mut output, "- {}", path.display())?;
    }

    utils::handle_output(output, &args.output_args, "Snapshot content")
}
