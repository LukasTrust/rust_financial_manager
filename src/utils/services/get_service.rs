use crate::utils::classes::get_utils::GetUtils;
use crate::utils::interfaces::i_get_utils::IGetUtils;
use once_cell::sync::Lazy;
use std::sync::{Arc, RwLock};

// Singleton instance of the utility service
static UTILS_SERVICE: Lazy<RwLock<Arc<dyn IGetUtils + Send + Sync>>> =
    Lazy::new(|| RwLock::new(Arc::new(GetUtils)));

// Accessor and mutator
pub fn set_get_service(service: Arc<dyn IGetUtils + Send + Sync>) {
    let mut utils = UTILS_SERVICE
        .write()
        .expect("Failed to acquire write lock on UTILS_SERVICE");
    *utils = service;
}

pub fn get_get_service() -> Arc<dyn IGetUtils + Send + Sync> {
    UTILS_SERVICE
        .read()
        .expect("Failed to acquire read lock on UTILS_SERVICE")
        .clone()
}
