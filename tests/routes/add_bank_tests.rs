#[cfg(test)]
mod tests {
    use rocket::{
        http::{ContentType, Cookie, Status},
        tokio,
    };

    use crate::test_help_functions::{get_loaded_user, get_test_client};

    #[tokio::test]
    async fn test_add_bank_view() {
        let client = get_test_client().await;

        let user = get_loaded_user().unwrap();

        let response = client
            .get("/add-bank")
            .private_cookie(Cookie::new("user_id", user.id.to_string()))
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Ok);
        let response_string = response.into_string().await.unwrap();

        assert!(response_string.contains("Add bank"));
        assert!(response_string.contains("Submit"));
    }

    #[tokio::test]
    async fn test_add_bank_form_success() {
        let client = get_test_client().await;

        let new_bank = "name=Test_Bank&link=http://test-bank.com&counterparty_column=0&amount_column=1&bank_balance_after_column=2&date_column=3";

        let user = get_loaded_user().unwrap();

        let response = client
            .post("/add-bank")
            .private_cookie(Cookie::new("user_id", user.id.to_string()))
            .header(ContentType::Form)
            .body(new_bank)
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Ok);
        let response_string = response.into_string().await.unwrap();

        assert!(response_string.contains("New bank added"));
        assert!(
            response_string.contains("The new bank 'Test_Bank' has been added to your profile.")
        );
    }

    #[tokio::test]
    async fn test_add_bank_form_failed_copy_bank() {
        let client = get_test_client().await;

        let new_bank = "name=copy_bank&link=http://test-bank.com&counterparty_column=0&amount_column=1&bank_balance_after_column=2&date_column=3";

        let response = client
            .post("/add-bank")
            .private_cookie(Cookie::new("user_id", "1"))
            .header(ContentType::Form)
            .body(new_bank)
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Ok);
        let response_string = response.into_string().await.unwrap();

        assert!(response_string.contains("Error inserting the bank"));
        assert!(response_string.contains(
            "A bank with this name already exists in your profile. Please choose a different bank name."
        ));
    }
}
