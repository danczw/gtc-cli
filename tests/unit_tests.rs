const KEY: &str = "openai_key";

#[cfg(test)]
mod tests {
    use dirs::home_dir;
    use gtc;
    use std::fs::File;
    use std::io::{self, Write};
    use std::path::PathBuf;

    use crate::KEY;

    // set_home_dir_path tests
    #[test]
    fn test_set_home_dir_path() {
        let file_name = "test.txt";
        let expected_path = home_dir().unwrap().join(file_name);
        assert_eq!(gtc::set_home_dir_path(file_name), expected_path);
    }

    #[test]
    fn test_set_home_dir_path_with_subdir() {
        let file_name = "test.txt";
        let subdir = "subdir";
        let expected_path = home_dir().unwrap().join(subdir).join(file_name);
        assert_eq!(
            gtc::set_home_dir_path(&format!("{}/{}", subdir, file_name)),
            expected_path
        );
    }

    // calc_hash tests
    #[test]
    fn test_calc_hash_success() {
        assert_eq!(gtc::calc_hash(&"hello"), gtc::calc_hash(&"hello"));
    }

    #[test]
    fn test_calc_hash_fail() {
        assert_ne!(gtc::calc_hash(&"foo"), gtc::calc_hash(&"bar"));
    }

    // read_context tests
    #[test]
    fn test_read_context() {
        let mut file = File::create(".test_read_context").unwrap();
        writeln!(file, "{}", KEY).unwrap();
        writeln!(file, "message 1").unwrap();
        writeln!(file, "message 2").unwrap();
        file.flush().unwrap();

        let context_file_path = PathBuf::from(".test_read_context");
        let expected_context = gtc::Context {
            openai_key: KEY.to_string(),
            key_hash: gtc::calc_hash(&KEY),
            hist: vec!["message 1".to_string(), "message 2".to_string()],
        };
        assert_eq!(gtc::read_context(&context_file_path), expected_context);

        std::fs::remove_file(".test_read_context").unwrap();
    }

    #[test]
    fn test_read_context_with_empty_file() {
        let mut file = File::create(".test_read_context_with_empty_file").unwrap();
        file.flush().unwrap();

        let context_file_path = PathBuf::from(".test_read_context_with_empty_file");
        let expected_context = gtc::Context {
            openai_key: "".to_string(),
            key_hash: gtc::calc_hash(&""),
            hist: vec![],
        };
        assert_eq!(gtc::read_context(&context_file_path), expected_context);
    }

    #[test]
    #[should_panic(expected = "No such file or directory")]
    fn test_read_context_with_invalid_file() {
        let context_file_path = PathBuf::from("/invalid/path");
        gtc::read_context(&context_file_path);
    }

    #[test]
    fn test_read_context_with_empty_key() {
        let mut file = File::create(".test_read_context_with_empty_key").unwrap();
        writeln!(file).unwrap();
        writeln!(file, "message 1").unwrap();
        writeln!(file, "message 2").unwrap();
        file.flush().unwrap();

        let context_file_path = PathBuf::from(".test_read_context_with_empty_key");
        let expected_context = gtc::Context {
            openai_key: "".to_string(),
            key_hash: gtc::calc_hash(&""),
            hist: vec!["message 1".to_string(), "message 2".to_string()],
        };
        assert_eq!(gtc::read_context(&context_file_path), expected_context);

        std::fs::remove_file(".test_read_context_with_empty_key").unwrap();
    }

    #[test]
    fn test_read_context_with_empty_history() {
        let mut file = File::create(".test_read_context_with_empty_history").unwrap();
        writeln!(file, "openai_key").unwrap();
        file.flush().unwrap();

        let context_file_path = PathBuf::from(".test_read_context_with_empty_history");
        let expected_context = gtc::Context {
            openai_key: KEY.to_string(),
            key_hash: gtc::calc_hash(&KEY),
            hist: vec![],
        };
        assert_eq!(gtc::read_context(&context_file_path), expected_context);

        std::fs::remove_file(".test_read_context_with_empty_history").unwrap();
    }

    // input tests
    #[test]
    fn test_input() {
        let mut writer = Vec::new();
        let reader = io::Cursor::new(b"yes");
        let result = gtc::input("Does this test pass?", reader, &mut writer).unwrap();
        assert_eq!(result, "yes");
        assert_eq!(writer, b"Does this test pass? ");
    }
    // TODO: test call_oai

    // test check_response
    #[tokio::test]
    async fn test_check_response_ok() {
        // Request a new server from the pool
        let mut server = mockito::Server::new_async().await;

        // Use one of these addresses to configure your client
        let url = server.url();

        // Create a mock response with status code 200 OK and some JSON data
        let _mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"foo": "bar"}"#)
            .create();

        // Create a new reqwest client and send a request to the mock server
        let client = reqwest::Client::new();
        let resp = client
            .post(&url)
            .body(r#"{"foo": "bar"}"#)
            .send()
            .await
            .unwrap();

        // Call the check_response function with the mock response
        let result = gtc::check_response(resp).await;

        // Assert that the function returns the expected JSON data
        assert_eq!(result.unwrap(), serde_json::json!({"foo": "bar"}));

        assert!(true)
    }

    #[tokio::test]
    async fn test_check_response_err() {
        // Request a new server from the pool
        let mut server = mockito::Server::new_async().await;

        // Use one of these addresses to configure your client
        let url = server.url();

        // Create a mock response with status code 200 OK and some JSON data
        let _mock = server
            .mock("POST", "/")
            .with_status(400)
            .with_header("content-type", "application/json")
            .with_body(r#"{"error": "bad request"}"#)
            .create();

        // Create a new reqwest client and send a request to the mock server
        let client = reqwest::Client::new();
        let resp = client
            .post(&url)
            .body(r#"{"foo": "bar"}"#)
            .send()
            .await
            .unwrap();

        // Call the check_response function with the mock response
        let result = gtc::check_response(resp).await;

        // Assert that the function returns the expected Error
        assert_eq!(
            result.unwrap_err().to_string(),
            r#"{"error": "bad request"}"#
        );

        assert!(true)
    }
}
