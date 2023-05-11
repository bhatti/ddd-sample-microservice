use crate::core::domain::Identifiable;
use crate::core::library::Role;

pub mod command;
pub mod domain;
pub mod dto;
pub mod factory;
pub mod controller;

pub(crate) trait Patron: Identifiable {
    fn is_admin(&self) -> bool;
    fn is_child(&self) -> bool;
    fn is_employee(&self) -> bool;
    fn is_librarian(&self) -> bool;
    fn is_role(&self, match_role: Role) -> bool;
    fn is_regular(&self) -> bool;
}
