use std::{env, fs, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Re-run if the template changes
    println!("cargo:rerun-if-changed=env.example");

    // Where to copy FROM (crate root)
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let env_example_path = manifest_dir.join(".env.example");

    // Compute target dir (your local data dir) and ensure it exists
    let mut out_dir = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    out_dir.push("sporlcli");
    fs::create_dir_all(&out_dir)?; // <-- create the actual directory, not only its parent

    // Only copy if the source exists; otherwise warn instead of failing
    if env_example_path.is_file() {
        let contents = fs::read_to_string(&env_example_path)?;
        fs::write(out_dir.join(".env.example"), contents)?;
    } else {
        println!(
            "cargo:warning=env.example not found at {}",
            env_example_path.display()
        );
        // If this should be fatal, replace with: return Err("env.example missing".into());
    }

    Ok(())
}
