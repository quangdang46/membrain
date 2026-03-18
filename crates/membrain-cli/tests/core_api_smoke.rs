#[test]
fn cli_depends_on_core_api() {
    let _: membrain_core::CoreApiVersion = membrain_cli::core_api_version();
}
