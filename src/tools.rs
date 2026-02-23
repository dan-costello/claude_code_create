use std::process::Command;

pub async fn read_file(file_path: String) -> Result<String, std::io::Error> {
    let contents = tokio::fs::read_to_string(file_path).await?;
    Ok(contents)
}

pub async fn write_file(file_path: String, contents: String) -> Result<(), std::io::Error> {
    tokio::fs::write(file_path, contents).await?;
    Ok(())
}

pub async fn execute_bash(command: String) -> Result<String, std::io::Error> {
    println!("Executing command: {}", command);
    // get cwd
    println!("{}", std::env::current_dir()?.display());
    let output = Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .expect("Failed to execute command");

        let combined = format!(
    "Stdout:{}\nStderr:{}",
    String::from_utf8_lossy(&output.stdout),
    String::from_utf8_lossy(&output.stderr)
);
    println!("{}", String::from_utf8_lossy(&output.stdout));
    return Ok(combined);
}
