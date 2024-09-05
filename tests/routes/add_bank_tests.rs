#[cfg(test)]
mod tests {
    use rocket::{
        http::{ContentType, Cookie, Status},
        tokio,
    };

    use crate::test_help_functions::get_test_client;

    #[tokio::test]
    async fn test_add_bank_view() {
        let client = get_test_client().await;

        let response = client
            .get("/add-bank")
            .private_cookie(Cookie::new("user_id", "0"))
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

        let response = client
            .post("/add-bank")
            .private_cookie(Cookie::new("user_id", "0"))
            .header(ContentType::Form)
            .body(new_bank)
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Ok);
        let response_string = response.into_string().await.unwrap();

        log::info!("{}", response_string);
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
            .private_cookie(Cookie::new("user_id", "0"))
            .header(ContentType::Form)
            .body(new_bank)
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Ok);
        let response_string = response.into_string().await.unwrap();

        assert!(response_string.contains("Bank already exists"));
        assert!(response_string.contains(
            "The bank 'copy_bank' could not be added because it already exists in your profile."
        ));
    }

    #[tokio::test]
    async fn test_add_bank_form_failed_csv_internal_error() {
        let client = get_test_client().await;

        let new_bank = "name=csv_error&link=http://test-bank.com&counterparty_column=0&amount_column=1&bank_balance_after_column=2&date_column=3";

        let response = client
            .post("/add-bank")
            .private_cookie(Cookie::new("user_id", "0"))
            .header(ContentType::Form)
            .body(new_bank)
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Ok);
        let response_string = response.into_string().await.unwrap();

        assert!(response_string.contains("Error inserting csv converter"));
        assert!(response_string.contains("The bank was added but the csv converter was not."));
    }

    #[tokio::test]
    async fn test_add_bank_form_failed_load_banks() {
        let client = get_test_client().await;

        let new_bank = "name=error_loading_banks&link=http://test-bank.com&counterparty_column=0&amount_column=1&bank_balance_after_column=2&date_column=3";

        let response = client
            .post("/add-bank")
            .private_cookie(Cookie::new("user_id", "0"))
            .header(ContentType::Form)
            .body(new_bank)
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Ok);
        let response_string = response.into_string().await.unwrap();

        assert!(response_string.contains("Error loading banks"));
        assert!(response_string.contains("There was an internal error trying to load the banks."));
    }
}
