#[cfg(test)]
mod tests {
    use rocket::{http::ContentType, http::Status, tokio};
    use rust_financial_manager::routes::register::{is_strong_password, is_valid_email};

    use crate::test_help_functions::get_test_client;

    #[tokio::test]
    async fn test_register_view() {
        let client = get_test_client().await;

        let response = client.get("/register").dispatch().await;

        assert_eq!(response.status(), Status::Ok);
    }

    #[tokio::test]
    async fn test_register_user_success() {
        let client = get_test_client().await;

        let new_user =
            "first_name=John&last_name=Doe&email=john.doe@mail.com&password=S3cureP@ssw0rd!";

        let response = client
            .post("/register")
            .header(ContentType::Form)
            .body(new_user)
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Ok);
        assert!(response
            .into_string()
            .await
            .unwrap()
            .contains("Registration successful. Please log in."));
    }

    #[tokio::test]
    async fn test_register_user_failed_copy_user() {
        let client = get_test_client().await;

        let new_user =
            "first_name=John&last_name=Doe&email=copy_email@mail.com&password=S3cureP@ssw0rd!";

        let response = client
            .post("/register")
            .header(ContentType::Form)
            .body(new_user)
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Ok);
        assert!(response
            .into_string()
            .await
            .unwrap()
            .contains("Email already exists. Please use a different email."));
    }

    #[tokio::test]
    async fn test_register_user_failed_invalid_email() {
        let client = get_test_client().await;

        let new_user = "first_name=John&last_name=Doe&email=invalid_email&password=S3cureP@ssw0rd!";

        let response = client
            .post("/register")
            .header(ContentType::Form)
            .body(new_user)
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Ok);
        assert!(response
            .into_string()
            .await
            .unwrap()
            .contains("Invalid email format. Please use a valid email."));
    }

    #[tokio::test]
    async fn test_register_user_failed_internal_error() {
        let client = get_test_client().await;

        let new_user =
            "first_name=John&last_name=Doe&email=internal_error@mail.com&password=S3cureP@ssw0rd!";

        let response = client
            .post("/register")
            .header(ContentType::Form)
            .body(new_user)
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Ok);
        assert!(response
            .into_string()
            .await
            .unwrap()
            .contains("Internal server error. Please try again later."));
    }

    #[test]
    fn test_is_valid_email() {
        // Positive cases
        assert!(is_valid_email("test@example.com"));
        assert!(is_valid_email("user.name+tag+sorting@example.com"));
        assert!(is_valid_email("user.name@example.co.uk"));
        assert!(is_valid_email("user_name@domain.com"));

        // Negative cases
        assert!(!is_valid_email("plainaddress"));
        assert!(!is_valid_email("@missingusername.com"));
        assert!(!is_valid_email("username@domain..com"));
        assert!(!is_valid_email("username@domain.c"));
        assert!(!is_valid_email("username@domain,com"));
    }

    #[test]
    fn test_is_strong_password() {
        // Positive cases
        assert!(is_strong_password("Str0ngP@ssw0rd!"));
        assert!(is_strong_password("Another$trongP4ss"));
        assert!(is_strong_password("P@ssw0rd12345"));

        // Negative cases
        assert!(!is_strong_password("short1A!"));
        assert!(!is_strong_password("alllowercase123!"));
        assert!(!is_strong_password("ALLUPPERCASE123!"));
        assert!(!is_strong_password("NoDigits!@#"));
        assert!(!is_strong_password("NoSpecialChars123"));
    }
}
