use membrain_daemon::daemon::DaemonRuntime;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let socket_path = "/tmp/membrain.sock";
    println!("Bootstrapping Membrain Daemon...");
    let mut runtime = DaemonRuntime::new(socket_path);
    runtime.run_until_stopped().await?;
    Ok(())
}
