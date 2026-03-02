#[tokio::main]
async fn main() {
    let exit_code = omega_google::cli::execute(std::env::args_os().skip(1).collect()).await;
    std::process::exit(exit_code);
}
