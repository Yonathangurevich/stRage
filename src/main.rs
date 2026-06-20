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
            .args(["diff", "--cached"])
            .output()?;

        // extracting the results
        if on_stage.status.success() {
            
            let output = String::from_utf8(on_stage.stdout)?;

            let system_prompt = "You are a senior Rust engineer acting as a mentor reviewing a git diff. \
                Your job is to make ME a better programmer, NOT to fix the code for me.\n\n\
                RULES (these override everything else):\n\
                - Do NOT write corrected code blocks, rewritten lines, or \"change X to Y\" fixes.\n\
                - You MAY drop a tiny hint: name a method, trait, type, or clippy lint, and give a \
                one-phrase nudge. But never the full solution — I should still have to think.\n\
                - Point me toward the problem and let me find the fix myself.\n\n\
                For each issue, output exactly:\n\
                  [SEVERITY] file:line — short title\n\
                  Why it matters: <one sentence>\n\
                  Think about: <a guiding question, may name a method/trait/lint as a hint>\n\
                  Learn more: <a Google search query I can paste>\n\n\
                SEVERITY is one of: BUG, SECURITY, SECRET, STYLE, IDIOM.\n\
                If you spot a leaked key, password, or token, flag it as SECRET FIRST, before anything else.\n\n\
                If a different crate or std feature would be more idiomatic, name it and give ONE sentence \
                on why it fits (no usage code), plus a search query so I go learn it myself.\n\n\
                Keep it under ~8 findings. If the diff is clean, say so in one line. \
                Review ONLY what changed in the diff.";

            let user_prompt = format!("Here is the git diff to review:\n\n{output}");

            // sending the request-prompt
            let response: Value = client
                .chat()
                .create_byot(json!({
                    "messages": [
                        {
                            "role": "system",
                            "content": system_prompt
                        },
                        {
                            "role": "user",
                            "content": user_prompt
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
        }

    }

    Ok(())
}
