use clap::error::Result;
use clap::{Arg, ArgAction, ArgMatches, Command};
use dirs::home_dir;
use reqwest::{header, Client, StatusCode};
use serde_json::{json, Map, Value};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::io;
use std::path::PathBuf;
use std::time::Duration;

pub fn set_home_dir_path(file_name: &str) -> PathBuf {
    let mut path = home_dir().unwrap();
    path.push(file_name);
    path
}

pub fn calc_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

#[derive(PartialEq, Debug)]
pub struct Context {
    pub openai_key: String,
    pub key_hash: u64,
    pub hist: Vec<String>,
}

pub fn read_context(hist_file_path: &PathBuf) -> Context {
    // Initialize Context struct
    let empt = String::from("");
    let mut ctx = Context {
        openai_key: empt.clone(),
        key_hash: calc_hash(&empt),
        hist: vec![],
    };
    // Read profile
    let saved = std::fs::read_to_string(hist_file_path).unwrap_or("".to_string());

    // if file is empty or doesn't exist, delete potential file return empty Context struct
    if saved.is_empty() {
        std::fs::remove_file(hist_file_path).unwrap();
        ctx
    } else {
        // get openai key from first line of file
        ctx.openai_key = saved.lines().next().unwrap().to_string();
        ctx.key_hash = calc_hash(&ctx.openai_key);

        // get chat history from rest of file
        for line in saved.lines().skip(1) {
            ctx.hist.push(line.to_string());
        }
        ctx
    }
}

pub fn new_context(key: String) -> Context {
    // Initialize Context struct
    let ctx = Context {
        openai_key: key.trim().to_string(),
        key_hash: calc_hash(&key),
        hist: vec![],
    };

    ctx
}

pub fn input<R, W>(prompt: &str, mut reader: R, mut writer: W) -> Result<String, io::Error>
where
    R: io::BufRead,
    W: io::Write,
{
    match write!(writer, "{} ", prompt) {
        Ok(_) => {}
        Err(e) => return Err(e),
    }
    writer.flush()?;

    let mut input = String::new();
    reader.read_line(&mut input)?;
    let input = input.trim();

    Ok(input.to_string())
}

pub fn cli() -> Command {
    Command::new("gtc")
        .about("A cli designed to facilitate seamless text-based conversations with ChatGPT.")
        .arg_required_else_help(true)
        .arg(
            Arg::new("message")
                .help("The message to send to ChatGPT in quotes.")
                // .short('m')
                // .long("message")
                .index(1)
                .action(ArgAction::Set)
                .required(true),
        )
    // TODO: arg to remove local context
    // TODO: arg to show local context
    // TODO: arg to show version
}

pub async fn call_oai(
    ctx: &Context,
    arg: &ArgMatches,
) -> Result<Value, Box<dyn std::error::Error>> {
    let new_msg = arg.get_one::<String>("message").unwrap();

    // Build the headers
    let mut headers = header::HeaderMap::new();
    headers.insert(
        "Content-Type",
        header::HeaderValue::from_static("application/json"),
    );
    let auth_value = format!("Bearer {}", ctx.openai_key.as_str());
    let mut auth_value = header::HeaderValue::from_str(&auth_value).unwrap();
    auth_value.set_sensitive(true);
    headers.insert(header::AUTHORIZATION, auth_value);

    // Build the URL
    let url = "https://api.openai.com/v1/chat/completions";

    // Build the body
    let mut body: Map<String, Value> = Map::new();
    body.insert("model".to_string(), json!("gpt-4"));

    let mut messages = Vec::new();
    for ctx_msg in ctx.hist.iter() {
        let role = ctx_msg.split("||").next().unwrap();
        let content = ctx_msg.split("||").nth(1).unwrap();
        messages.push(json!({"role": role, "content": content}));
    }
    messages.push(json!({"role": "user", "content": new_msg}));
    body.insert("messages".to_string(), Value::Array(messages));
    let body_json = Value::Object(body);

    // Initialize client and send request
    let client = Client::builder()
        .default_headers(headers)
        .timeout(Duration::new(120, 0))
        .build()?;
    let resp = client.post(url).json(&body_json).send().await?;

    check_response(resp).await
}

pub async fn check_response(resp: reqwest::Response) -> Result<Value, Box<dyn std::error::Error>> {
    // Get response values
    let resp_status = resp.status();
    // Deserialize response text
    let resp_text = resp.text().await?;
    let resp_json: Value = serde_json::from_str(&resp_text)?;

    // check response
    match resp_status {
        StatusCode::OK => {
            // return response text
            Ok(resp_json)
        }
        _ => {
            // return error message
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                resp_text,
            )))
        }
    }
}
