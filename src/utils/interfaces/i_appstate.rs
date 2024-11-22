use crate::utils::{
    classes::language::Language,
    structs::{Bank, ErrorResponse},
};
use rocket::serde::json::Json;

#[async_trait::async_trait]
pub trait IAppState {
    async fn get_current_bank(
        &self,
        cookie_user_id: i32,
        cookie_use_language: Language,
    ) -> Result<Bank, Json<ErrorResponse>>;

    async fn set_current_bank(&self, cookie_user_id: i32, current_bank: Option<Bank>);
}
