use clap::Parser;
use env_logger::Env;
use log::{error, info};

use std::fs;
use std::path::Path;
use std::process::exit;

use crate::workflow::do_workflow;

mod workflow;

#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
/// Prepare, run and collect iguana containers based on passed iguana workflow file
struct Args {
   /// File or URL with iguana workflow
   #[clap(short = 'f', long, value_parser, default_value = "control.yaml")]
   workflow: String,

   /// Newroot mount directory
   #[clap(short, long, value_parser, default_value = "/sysroot")]
   newroot: String,
}

/// Tracking results of individual job runs

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let args = Args::parse();

    let workflow_file = args.workflow;
    // Is workflow URL or file
    info!("Using workflow file {}", workflow_file);
    if !Path::is_file(Path::new(&workflow_file)) {
        error!("No such file: {}", workflow_file);
        exit(1);
    }

    let workflow_data = fs::read_to_string(workflow_file).expect("Unable to open workflow file");
    if let Err(e) = do_workflow(workflow_data) {
        error!("{}", e);
        exit(1);
    } else {
        info!("Iguana workflow finished successfuly");
        exit(0);
    }
}
