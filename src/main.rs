use clap::Parser;
use std::process::Command;
use std::{error, env, process};
use async_openai::{Client, config::OpenAIConfig};
use serde_json::{json, Value};

#[derive(Parser)]
#[command(name = "stRage")]
#[command(version = "1.0")]
#[command(about = "AI review", long_about = None)]
struct Cli {
    #[arg(short, long)]
    review: bool
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    let args = Cli::parse();

    // connect api-key
    dotenvy::dotenv().ok();

    let api_key = env::var("OPENROUTER_API_KEY").unwrap_or_else(|_| {
        eprintln!("NO API-KEY WAS FOUND");
        process::exit(1);
    });

    let base_url = String::from("https://openrouter.ai/api/v1");

    let config = OpenAIConfig::new()
        .with_api_base(base_url)
        .with_api_key(api_key);

    let client = Client::with_config(config);
    
    // checking the review flag
    if args.review {
        
        // running the command for cached
        let on_stage = Command::new("git")
            .arg("diff")
            .arg("--cached")
            .output()
            .expect("failed to execute process");

        // extracting the results
        if on_stage.status.success() {
            if let Ok(output) = String::from_utf8(on_stage.stdout) {
                // println!("the output: {}", output);

                let base_prompt = format!(
                    "Hi, you are code reviewer, based on this last code: {output} 
                    could you find any bugs or security holes?
                    make the answer as short as possible"
                );

                // sending the request-prompt
                let response: Value = client
                    .chat()
                    .create_byot(json!({
                        "messages": [
                            {
                                "role": "user",
                                "content": base_prompt
                            }
                        ],
                        "model": "cohere/north-mini-code:free",
                    }))
                    .await?;

                if let Some(message) = response["choices"][0].as_object() {
                    if let Some(content) = message["message"].as_object() {
                        if let Some(llm_respond) = content["content"].as_str() {
                            println!("{}", llm_respond);
                        }
                    }
                }
                // println!("{:#?}", response);
            }
        }

    }

    Ok(())
}
