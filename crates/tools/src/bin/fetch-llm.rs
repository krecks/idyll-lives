//! `cargo fetch-llm`: see `ll_tools::llmfetch`.
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    std::process::exit(ll_tools::llmfetch::main_with(&args));
}
