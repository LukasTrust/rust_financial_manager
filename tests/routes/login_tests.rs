#[cfg(test)]
mod tests {
    use rocket::{http::ContentType, http::Status, tokio};

    use crate::test_help_functions::get_test_client;

    #[tokio::test]
    async fn test_login_view() {
        let client = get_test_client().await;

        let response = client.get("/").dispatch().await;

        assert_eq!(response.status(), Status::Ok);
    }

    #[tokio::test]
    async fn test_login_view_success() {
        let client = get_test_client().await;

        let response = client
            .get("/login?success=Registration&Success")
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Ok);
    }

    #[tokio::test]
    async fn test_login_user_success() {
        let client = get_test_client().await;

        let user = "email=user_exists@mail.com&password=Password123";

        let response = client
            .post("/login")
            .header(ContentType::Form)
            .body(user)
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Ok);

        let response_string = response.into_string().await.unwrap();

        assert!(response_string.contains("Login successful. Redirecting..."));
    }

    #[tokio::test]
    async fn test_login_user_failed_password_not_matching() {
        let client = get_test_client().await;

        let user = "email=wrong_password@mail.com&password=test";

        let response = client
            .post("/login")
            .header(ContentType::Form)
            .body(user)
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Ok);

        let response_string = response.into_string().await.unwrap();

        assert!(
            response_string.contains("Login failed. Either the email or password was incorrect."),
        );
    }

    #[tokio::test]
    async fn test_login_user_failed_email_not_found() {
        let client = get_test_client().await;

        let user = "email=fake_email@mail.com&password=Password123";

        let response = client
            .post("/login")
            .header(ContentType::Form)
            .body(user)
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Ok);
        assert!(response
            .into_string()
            .await
            .unwrap()
            .contains("Login failed. Either the email or password was incorrect."),);
    }
}
