use crate::helpers::spawn_app;

// `tokio::test` is the testing equivalent of `tokio::main`
// It will convert an async function into a synchronous test
// It also spares you from having to specify the `#[test]` attribute
//
// You can inspect what code gets generated with:
// `cargo expand --test health_check` (<- name of the test file)
#[tokio::test]
async fn health_check_works() {
    // Arrange
    let app = spawn_app().await;

    // We need to bring in `reqwest`
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(format!("{}/health_check", &app.address))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}
