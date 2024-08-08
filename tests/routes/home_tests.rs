#[cfg(test)]
mod tests {
    use rocket::http::{Cookie, Status};

    use crate::test_help_functions::test_client;

    #[rocket::async_test]
    async fn test_base_without_cookies() {
        let client = test_client().await;
        let response = client.get("/base").dispatch().await;

        assert_eq!(response.status(), Status::SeeOther);
    }

    #[rocket::async_test]
    async fn test_base_with_valid_user_id_cookie() {
        let client = test_client().await;
        let response = client
            .get("/base")
            .cookie(Cookie::new("user_id", "123"))
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Ok);
    }

    #[rocket::async_test]
    async fn test_base_with_invalid_user_id_cookie() {
        let client = test_client().await;
        let response = client
            .get("/base")
            .cookie(Cookie::new("user_id", "invalid"))
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::SeeOther);
    }
}
