use structopt::StructOpt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::error::Error;
use Justice::CheckInResponse;

#[derive(StructOpt, Debug)]
#[structopt(name = "prisonyard-cli", about = "CLI to interact with the operator API in PrisonYard")]
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
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let opt = Opt::from_args();
    let client = Client::new();
    let base_url = "http://localhost:8000"; // Adjust the base URL as needed

    match opt {
        Opt::ListInmates => {
            let response = client.get(&format!("{}/operator", base_url))
                .send()
                .await?
                .text()
                .await?;
            println!("{}", response);
        }
        Opt::GetInmate { implant_id } => {
            let response = client.get(&format!("{}/operator/{}", base_url, implant_id))
                .send()
                .await?
                .text()
                .await?;
            println!("{}", response);
        }
        Opt::GetRecentTask { implant_id } => {
            let response = client.get(&format!("{}/operator/{}/recent", base_url, implant_id))
                .send()
                .await?;
            if response.status().is_success() {
                let task: CheckInResponse = response.json().await?;
                println!("{:?}", task);
            } else {
                let error = response.text().await?;
                println!("Error: {}", error);
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
