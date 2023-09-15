use tokio::{fs::File, io::AsyncReadExt};

pub const DEFAULT_FOLDER: &str = "./wwwroot";

pub async fn get(filename: &str) -> std::io::Result<Vec<u8>> {
    let mut file = File::open(filename).await?;

    let mut result: Vec<u8> = Vec::new();

    loop {
        let res = file.read_buf(&mut result).await?;

        if res == 0 {
            break;
        }
    }

    return Ok(result);
}
