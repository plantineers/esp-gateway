use serde::{Deserialize, Serialize};
#[derive(Deserialize, Serialize, Debug)]
struct SensorData {
    controller: [char; 32],
    sensors: Vec<Data>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Data {
    r#type: String,
    value: f32,
}

#[tokio::main]
async fn main() {
    println!("Hello, world!");
}

async fn publish_data(data: SensorData) -> reqwest::Result<()>{
    Ok(())
}

#[cfg(test)]
mod tests{
    use super::*;
    use std::fs::read_to_string;

    #[test]
    fn test_ron_decoding(){
        let data = read_to_string("testdata.json").unwrap();
        let decoded: SensorData = serde_json::from_str(&data).unwrap();
        println!("{:?}", decoded);
    }
}