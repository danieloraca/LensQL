use super::model::ConnectionProfile;
use crate::errors::AppError;

pub trait ConnectionRepo {
    fn load_connections(&self) -> Result<Vec<ConnectionProfile>, AppError>;
    fn save_connections(&self, conns: &[ConnectionProfile]) -> Result<(), AppError>;
}
