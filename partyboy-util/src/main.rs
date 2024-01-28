use clap::Parser;

mod convert_junit_to_md;
mod gen_bios_skip_snapshot;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
enum CliCommands {
    GenerateBiosSkipSnapshot(gen_bios_skip_snapshot::Args),
    ConvertJunitToMd(convert_junit_to_md::Args),
}

fn main() {
    match CliCommands::parse() {
        CliCommands::GenerateBiosSkipSnapshot(args) => gen_bios_skip_snapshot::execute(args),
        CliCommands::ConvertJunitToMd(args) => convert_junit_to_md::execute(args),
    }
}
