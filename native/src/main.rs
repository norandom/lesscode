use clap::Parser;

#[derive(Parser)]
#[command(
    version,
    about = "Native capability-compression core (foundation build)"
)]
#[command(
    after_help = "This build provides parser foundations. Prepare/compare commands are not implemented yet."
)]
struct Args {}

fn main() {
    Args::parse();
}
