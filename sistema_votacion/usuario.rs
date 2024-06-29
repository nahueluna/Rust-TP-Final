use ink::prelude::string::String;

/// Información personal del usuario que integra el sistema
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
#[derive(Debug, Clone)]
pub struct Usuario {
    pub nombre: String,
    pub apellido: String,
    pub dni: String,
}

impl Usuario {
    /// Creacion de un usuario con su información personal
    pub fn new(nombre: String, apellido: String, dni: String) -> Self {
        Self { nombre, apellido, dni }
    }
}
