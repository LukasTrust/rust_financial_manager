#[cfg(test)]
mod tests {
    use rocket::http::Status;

    use rust_financial_manager::{database::models::NewUser, utils::structs::FormUser};

    use crate::test_help_functions::{test_client, user_login, user_register};

    #[rocket::async_test]
    async fn test_login_form() {
        let client = test_client().await;
        let response = client.get("/").dispatch().await;

        assert_eq!(response.status(), Status::Ok);
    }

    #[rocket::async_test]
    async fn test_login_user_success() {
        let client = test_client().await;

        let email_for_test = "bob.doe@example.com";
        let password_for_test = "Str0ngP@ssw0rd";

        // Register a user
        let register_form = NewUser {
            first_name: "John".into(),
            last_name: "Doe".into(),
            email: email_for_test.into(),
            password: password_for_test.into(),
        };

        let response = user_register(&client, register_form).await;
        assert_eq!(response.status(), Status::SeeOther);

        // Attempt login with correct credentials
        let login_form = FormUser {
            email: email_for_test.to_string(),
            password: password_for_test.to_string(),
        };

        let response = user_login(&client, login_form).await;
        assert_eq!(response.status(), Status::SeeOther);

        // Cleanup
        let delete_response = client
            .delete(format!("/delete_user/{}", email_for_test))
            .dispatch()
            .await;

        assert_eq!(delete_response.status(), Status::SeeOther);
    }

    #[rocket::async_test]
    async fn test_login_user_not_exist() {
        let client = test_client().await;

        let email_for_test = "not.user@example.com";
        let password_for_test = "Str0ngP@ssw0rd";

        let login_form = FormUser {
            email: email_for_test.to_string(),
            password: password_for_test.to_string(),
        };

        let response = user_login(&client, login_form).await;
        let body = response.into_string().await.unwrap();
        assert!(body.contains("Login failed. Either the email or password was incorrect."));
    }

    #[rocket::async_test]
    async fn test_login_user_wrong_password() {
        let client = test_client().await;

        let email_for_test = "wrong.password@example.com";
        let password_for_test = "Str0ngP@ssw0rd";

        // Register a user
        let register_form = NewUser {
            first_name: "John".into(),
            last_name: "Doe".into(),
            email: email_for_test.into(),
            password: password_for_test.into(),
        };

        let response = user_register(&client, register_form).await;
        assert_eq!(response.status(), Status::SeeOther);

        // Attempt login with incorrect password
        let login_form = FormUser {
            email: email_for_test.to_string(),
            password: "OtherPassword".to_string(),
        };

        let response = user_login(&client, login_form).await;

        let body = response.into_string().await.unwrap();
        assert!(body.contains("Login failed. Either the email or password was incorrect."));

        // Cleanup
        let delete_response = client
            .delete(format!("/delete_user/{}", email_for_test))
            .dispatch()
            .await;

        assert_eq!(delete_response.status(), Status::SeeOther);
    }

    #[rocket::async_test]
    async fn test_sql_injection() {
        let client = test_client().await;

        // Register a user with a strong password
        let email_for_test = "injection.test@example.com";
        let strong_password = "Str0ngP@ssw0rd123";

        let register_form = NewUser {
            first_name: "John".into(),
            last_name: "Doe".into(),
            email: email_for_test.into(),
            password: strong_password.into(),
        };

        let response = user_register(&client, register_form).await;
        assert_eq!(response.status(), Status::SeeOther);

        // Attempt SQL injection in the login attempt
        let sql_injection_payload = "password' OR '1'='1"; // Common SQL injection attempt

        let login_form = FormUser {
            email: email_for_test.to_string(),
            password: sql_injection_payload.to_string(),
        };

        let response = user_login(&client, login_form).await;

        let body = response.into_string().await.unwrap();
        assert!(body.contains("Login failed. Either the email or password was incorrect."));

        // Cleanup
        let delete_response = client
            .delete(format!("/delete_user/{}", email_for_test))
            .dispatch()
            .await;

        assert_eq!(delete_response.status(), Status::SeeOther);
    }
}
