#[cfg(test)]
mod tests {
    use rocket::{
        http::{Cookie, Status},
        tokio,
    };

    use crate::test_help_functions::get_test_client;

    #[tokio::test]
    async fn test_base_view_no_cookie() {
        let client = get_test_client().await;

        let response = client.get("/base").dispatch().await;

        assert_eq!(response.status(), Status::SeeOther);
    }

    #[tokio::test]
    async fn test_base_view_with_cookie() {
        let client = get_test_client().await;

        let response = client
            .get("/base")
            .private_cookie(Cookie::new("user_id", "0"))
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Ok);
        let response_string = response.into_string().await.unwrap();

        assert!(response_string.contains("Dashboard"));
        assert!(response_string.contains("Add new bank"));
        assert!(response_string.contains("Logout"));
    }

    #[tokio::test]
    async fn test_dashoard_view() {
        let client = get_test_client().await;

        let response = client
            .get("/dashboard")
            .private_cookie(Cookie::new("user_id", "0"))
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Ok);
        let response_string = response.into_string().await.unwrap();

        assert!(response_string.contains("Welcome, John Doe!"));
        assert!(response_string.contains("Number of transaction:"));
        assert!(response_string.contains("Number of contracts:"));
    }

    #[tokio::test]
    async fn test_settings_view() {
        let client = get_test_client().await;

        let response = client
            .get("/settings")
            .private_cookie(Cookie::new("user_id", "0"))
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Ok);
        let response_string = response.into_string().await.unwrap();

        assert!(response_string.contains("Settings"));
    }

    #[tokio::test]
    async fn test_logout_view() {
        let client = get_test_client().await;

        let response = client
            .get("/logout")
            .private_cookie(Cookie::new("user_id", "0"))
            .dispatch()
            .await;

        assert!(client.cookies().get_private("user_id").is_none());
        assert!(response.status() == Status::Ok);
        assert!(response.into_string().await.unwrap().contains("Login"));
    }
}
