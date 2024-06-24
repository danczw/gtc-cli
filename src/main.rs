use colored::*;
use std::io::{self, Write};

#[tokio::main]
async fn main() {
    // TODO: add logging

    const GTC_PROFILE: &str = ".gtc";
    let gtc_profile_path = gtc::set_home_dir_path(GTC_PROFILE);

    // parse command line arguments
    let matches = gtc::cli().get_matches();

    if matches.contains_id("message") {
        let mut ctx = if gtc_profile_path.exists() {
            // read existing profile file
            let mut ctx_read = gtc::read_context(&gtc_profile_path);

            match ctx_read.openai_key.is_empty() {
                // prompt user for openai key
                true => {
                    // get openai key from user
                    let openai_key = gtc::input(
                        "No OpenAI API key found, please enter:",
                        &mut io::stdin().lock(),
                        &mut io::stdout(),
                    );
                    // update context and return
                    ctx_read.openai_key = openai_key.unwrap().trim().to_string();
                    ctx_read.hist = vec![];
                    ctx_read
                }
                // else return context
                false => ctx_read,
            }
        } else {
            // create profile if it doesn't exist and prompt user for openai key
            let key_input = gtc::input(
                "No OpenAI API key found, please enter:",
                &mut io::stdin().lock(),
                &mut io::stdout(),
            );
            let openai_key = key_input.unwrap().trim().to_string();

            // update context and return
            gtc::Context {
                openai_key: openai_key.clone(),
                key_hash: gtc::calc_hash(&openai_key),
                hist: vec![],
            }
        };

        // call OpenAI API and display response
        let oai_response = gtc::call_oai(&ctx, &matches).await;
        match oai_response {
            Ok(resp_value) => {
                let answer = resp_value["choices"][0]["message"]["content"]
                    .as_str()
                    .unwrap();
                println!("{}", answer.cyan());
                // add message and answer to chat history
                ctx.hist.push(
                    "user||".to_owned() + matches.get_one::<String>("message").unwrap().as_str(),
                );
                ctx.hist.push("assistant||".to_owned() + answer);
                // clear profile file and write key as well as last 6 messages to file
                let mut file = std::fs::File::create(&gtc_profile_path).unwrap();
                writeln!(file, "{}", ctx.openai_key).unwrap();
                for line in ctx.hist.iter().rev().take(6).rev() {
                    writeln!(file, "{}", line.replace('\n', "")).unwrap();
                }
            }
            Err(e) => println!("{}", e),
        }
    }

    // add
}
