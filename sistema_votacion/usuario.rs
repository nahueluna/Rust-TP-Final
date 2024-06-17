use ink::prelude::string::String;

/*
 * Informacion general de votantes y candidatos, almacenado en el sistema
 */
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
#[derive(Debug)]
pub(crate) struct Usuario {
    hash: String,
    nombre: String,
    apellido: String,
    dni: u32,
    validado: bool,
}

impl Usuario {
    // Creacion de usuario (votante o candidato)
    pub(crate) fn new(hash: String, nombre: String, apellido: String, dni: u32) -> Self {
        Self {
            hash,
            nombre,
            apellido,
            dni,
            validado: false,
        }
    }
}
