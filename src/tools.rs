pub async fn read_file(file_path: String) -> Result<String, std::io::Error> {
    let contents = tokio::fs::read_to_string(file_path).await?;
    Ok(contents)
}

pub async fn write_file(file_path: String, contents: String) -> Result<(), std::io::Error> {
    tokio::fs::write(file_path, contents).await?;
    Ok(())
}