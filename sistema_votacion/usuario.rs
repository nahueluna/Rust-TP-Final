use ink::{prelude::string::String, primitives::AccountId};

/*
 * Informacion general de votantes y candidatos, almacenado en el sistema
 */
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
#[derive(Debug)]
pub(crate) struct Usuario {
    id: AccountId,
    nombre: String,
    apellido: String,
}

impl Usuario {
    // Creacion de usuario (votante o candidato)
    pub(crate) fn new(id: AccountId, nombre: String, apellido: String) -> Self {
        Self {
            id,
            nombre,
            apellido,
        }
    }
}
