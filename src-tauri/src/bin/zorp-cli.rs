//! Headless command-line runner: `zorp-cli run <folder> [options]`.
fn main() {
    std::process::exit(zorp_lib::cli::main())
}
