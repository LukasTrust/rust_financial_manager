#[cfg(test)]
mod tests {
    use rocket::http::{ContentType, Status};
    use rocket::local::asynchronous::Client;
    use rocket::{routes, Build, Rocket};
    use rocket_db_pools::Database;
    use rocket_dyn_templates::Template;
    use rust_financial_manager::database::db_connector::DbConn;
    use rust_financial_manager::database::models::NewUser;
    use rust_financial_manager::routes::delete_user::delete_user;
    use rust_financial_manager::routes::register::{is_strong_password, is_valid_email};
    use rust_financial_manager::routes::register::{
        login_form_from_register, register_form, register_user,
    };
    use urlencoding::encode;

    // Helper function to create a Rocket instance for testing
    fn rocket() -> Rocket<Build> {
        rocket::build()
            .mount(
                "/",
                routes![
                    login_form_from_register,
                    register_form,
                    register_user,
                    delete_user
                ],
            )
            .attach(Template::fairing())
            .attach(DbConn::init())
    }

    // Helper function to create a test client asynchronously
    async fn test_client() -> Client {
        Client::tracked(rocket())
            .await
            .expect("valid rocket instance")
    }

    fn form_encoded(body: &NewUser) -> String {
        format!(
            "firstname={}&lastname={}&email={}&password={}",
            encode(&body.first_name),
            encode(&body.last_name),
            encode(&body.email),
            encode(&body.password)
        )
    }

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
        // Initialize Rocket client
        let client = test_client().await;

        let email_for_test = "john.doe@example.com";

        // Prepare a new user registration form
        let form = NewUser {
            first_name: "John".into(),
            last_name: "Doe".into(),
            email: email_for_test.into(),
            password: "Str0ngP@ssw0rd".into(),
        };

        let form_body = form_encoded(&form); // Serialize the form to x-www-form-urlencoded format

        // Send POST request to /register
        let response = client
            .post("/register")
            .header(ContentType::Form) // Set content type to Form
            .body(form_body) // Set the serialized form body
            .dispatch()
            .await; // Await the response

        // Assert that the response status is a redirect (See Other)
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

        let initial_form = NewUser {
            first_name: "Jane".into(),
            last_name: "Doe".into(),
            email: email_for_test.into(),
            password: "InitialP@ssw0rd".into(),
        };

        let initial_form_body = form_encoded(&initial_form);

        let response = client
            .post("/register")
            .header(ContentType::Form)
            .body(initial_form_body)
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::SeeOther);

        let duplicate_form = NewUser {
            first_name: "Jane".into(),
            last_name: "Doe".into(),
            email: email_for_test.into(),
            password: "AnotherP@ssw0rd".into(),
        };

        let duplicate_form_body = form_encoded(&duplicate_form);

        let response = client
            .post("/register")
            .header(ContentType::Form)
            .body(duplicate_form_body)
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Ok);
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

    #[test]
    fn test_valid_email() {
        assert!(is_valid_email("test@example.com"));
        assert!(is_valid_email("user.name@domain.co"));
        assert!(is_valid_email("user-name@domain.com"));
    }

    #[test]
    fn test_invalid_email() {
        assert!(!is_valid_email("plainaddress"));
        assert!(!is_valid_email("user@domain"));
        assert!(!is_valid_email("user@domain..com"));
        assert!(!is_valid_email("user@domain.c"));
    }

    #[test]
    fn test_strong_password() {
        assert!(is_strong_password("StrongP@ssw0rd"));
        assert!(is_strong_password("1A!aB2#bC3$dE"));
    }

    #[test]
    fn test_weak_password() {
        assert!(!is_strong_password("weakpassword"));
        assert!(!is_strong_password("short1A!"));
        assert!(!is_strong_password("NoSpecialChar1"));
    }
}
