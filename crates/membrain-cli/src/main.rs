fn main() {
    let version = membrain_cli::core_api_version();
    println!(
        "membrain CLI bootstrap (core API {}.{})",
        version.major, version.minor
    );
}
