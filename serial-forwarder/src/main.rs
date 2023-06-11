use std::process::Stdio;
use tokio::{io::AsyncBufReadExt, task};
use serde::{Deserialize, Serialize};
use tokio::{process::Command, io::BufReader};
#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct SensorData {
    controller: [char; 32],
    sensors: Vec<Data>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Data {
    r#type: String,
    value: f32,
}

/// The Server Data format differs from the SensorData format, which is why we need to convert it. This is the format the server expects.
/// Since the server doesnt rewrite the request we have to rewrite it into their format here
/// The format is as follows:
/// ```json
/// {
///    "data": [
///        {
///            "controller": "a955f72e-1e90-492f-bc62-a2145dd39f38",
///            "sensor": "temperature",
///           "value": 20.7
///        }
///    ]
///}
/// ```
#[derive(Debug, Serialize, Deserialize)]
struct ServerData {
    data: Vec<ServerDataEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ServerDataEntry {
    controller: String,
    sensor: String,
    value: f32,
}

impl From<SensorData> for ServerData {
    fn from(data: SensorData) -> Self {
        let mut server_data = ServerData {
            data: Vec::new(),
        };
        for sensor in data.sensors {
            server_data.data.push(ServerDataEntry {
                /// We have to insert '-' into the UUID
                controller: format!(
                    "{}-{}-{}-{}-{}",
                    &data.controller[0..8].iter().collect::<String>(),
                    &data.controller[8..12].iter().collect::<String>(),
                    &data.controller[12..16].iter().collect::<String>(),
                    &data.controller[16..20].iter().collect::<String>(),
                    &data.controller[20..32].iter().collect::<String>(),
                ),
                sensor: sensor.r#type,
                value: sensor.value,
            });
        }
        server_data
    }
}

/// Spawn an espmonitor and monitor it's output. When it outputs data, publish it to the server in the server data format
#[tokio::main]
async fn main() {
    let mut binding = Command::new("espmonitor");
    let cmd = binding.arg("/dev/ttyUSB0");
    cmd.stdout(Stdio::piped());
    let mut child = cmd.spawn().unwrap();
    let stdout = child.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout).lines();

    while let Some(line) = reader.next_line().await.unwrap() {
        if line.starts_with("Decoded:") {
            let data: Result<SensorData, serde_json::Error> = serde_json::from_str(&line[8..]);
            // We do not want to block on network requests 
            if data.is_err() {
                println!("Error decoding data: {}", data.err().unwrap());
                continue;
            }
            task::spawn(async move {
                // For efficiencys sake we could use a single client for all requests, but that would involve an Arc<Mutex<>> and the gain is too little to justify the complexity
                let mut client = reqwest::Client::new();
                let server_data = data.unwrap().into();
                match publish_data(&mut client, &server_data).await {
                    Ok(_) => println!("Published data {:?}", server_data),
                    Err(e) => println!("Error publishing data: {}", e),
                }
            });
        }
    }


}

async fn publish_data(client: &mut reqwest::Client, data: &ServerData) -> reqwest::Result<()>{
    let res = client.post(format!("{}sensor-data", std::env::var("ENDPOINT").expect("You need to set the ENDPOINT Environment variable to the server address(e.g. http://localhost/v1/").to_string()))
        .header("Content-Type", "application/json")
        .header("Authentication", format!("Basic {}", std::env::var("AUTH").expect("You need to set the AUTH environment to a Basic Auth Value").to_string()))
        .json(data)
        .send()
        .await?;
    res.error_for_status()?;
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

    #[tokio::test]
    async fn test_publish_data(){
        let data = SensorData{
            controller: ['t'; 32],
            sensors: vec![Data{
                r#type: "test".to_string(),
                value: 1.0,
            }],
        };
        let mut client = reqwest::Client::new();
        let res = publish_data(&mut client, data).await.unwrap();
    }
}