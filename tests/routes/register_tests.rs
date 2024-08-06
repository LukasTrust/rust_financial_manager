#[cfg(test)]
mod tests {
    use rocket::http::Status;

    use rust_financial_manager::database::models::NewUser;
    use rust_financial_manager::routes::register::{is_strong_password, is_valid_email};

    use crate::test_help_functions::{test_client, user_register};

    #[rocket::async_test]
    async fn test_login_form_from_register() {
        let client = test_client().await;
        let response = client.get("/login?message=TestMessage").dispatch().await;

        assert_eq!(response.status(), Status::Ok);
        let body = response.into_string().await.unwrap();
        assert!(body.contains("TestMessage"));
    }

    #[rocket::async_test]
    async fn test_register_form() {
        let client = test_client().await;
        let response = client.get("/register").dispatch().await;

        assert_eq!(response.status(), Status::Ok);
    }

    #[rocket::async_test]
    async fn test_register_and_delete_user_success() {
        let client = test_client().await;

        let email_for_test = "john.doe@example.com";

        // Prepare a new user registration form
        let form = NewUser {
            first_name: "John".into(),
            last_name: "Doe".into(),
            email: email_for_test.into(),
            password: "Str0ngP@ssw0rd".into(),
        };

        let response = user_register(&client, form).await;

        assert_eq!(response.status(), Status::SeeOther);

        // Prepare a delete request to remove the user
        let delete_response = client
            .delete(format!("/delete_user/{}", email_for_test))
            .dispatch()
            .await;

        // Assert that the delete response status is a redirect (See Other)
        assert_eq!(delete_response.status(), Status::SeeOther);
    }

    #[rocket::async_test]
    async fn test_register_user_existing_email() {
        let client = test_client().await;

        let email_for_test = "unique.email@example.com";

        let form = NewUser {
            first_name: "Jane".into(),
            last_name: "Doe".into(),
            email: email_for_test.into(),
            password: "InitialP@ssw0rd".into(),
        };

        let response = user_register(&client, form).await;

        assert_eq!(response.status(), Status::SeeOther);

        let duplicate_form = NewUser {
            first_name: "Jane".into(),
            last_name: "Doe".into(),
            email: email_for_test.into(),
            password: "AnotherP@ssw0rd".into(),
        };

        let response = user_register(&client, duplicate_form).await;

        let body = response.into_string().await.unwrap();
        assert!(body.contains("Email already exists"));

        // Prepare a delete request to remove the user
        let delete_response = client
            .delete(format!("/delete_user/{}", email_for_test))
            .dispatch()
            .await;

        // Assert that the delete response status is a redirect (See Other)
        assert_eq!(delete_response.status(), Status::SeeOther);
    }

    #[rocket::async_test]
    async fn test_register_user_weak_password() {
        let client = test_client().await;

        let email_for_test = "weak.email@example.com";

        let form = NewUser {
            first_name: "Jane".into(),
            last_name: "Doe".into(),
            email: email_for_test.into(),
            password: "weak".into(),
        };

        let response = user_register(&client, form).await;

        let body = response.into_string().await.unwrap();
        assert!(body.contains("Password must be at least 10 characters long and contain at least one uppercase letter, one lowercase letter, one digit, and one special character"));
    }
}
