use std::io::Write;
use std::process::Command;

type Result<T> = std::result::Result<T, failure::Error>;

pub fn build() -> Result<()> {
    let output = Command::new("npm").args(&["run", "build"]).output()?;
    println!("status: {}", output.status);
    std::io::stdout().write_all(&output.stdout).unwrap();
    std::io::stderr().write_all(&output.stderr).unwrap();
    Ok(())
}
