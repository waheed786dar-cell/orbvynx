//! ORBVYNX CLI — entry point
//! Boot lifecycle: see docs/architecture/part-2b-boot-lifecycle.md

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    tracing::info!("ORBVYNX booting...");

    orbvynx_kernel::placeholder();

    println!("ORBVYNX kernel skeleton is alive.");
    Ok(())
}
