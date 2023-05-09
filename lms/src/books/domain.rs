use crate::core::domain::Identifiable;
use crate::core::library::BookStatus;

pub mod model;

pub(crate) trait Book: Identifiable {
    fn is_restricted(&self) -> bool;
    fn status(&self) -> BookStatus;
}
