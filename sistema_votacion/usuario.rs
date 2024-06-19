use ink::prelude::string::String;

/*
 * Informacion general de votantes y candidatos, almacenado en el sistema
 */
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
#[derive(Debug)]
pub(crate) struct Usuario {
    nombre: String,
    apellido: String,
}

impl Usuario {
    // Creacion de usuario (votante o candidato)
    pub(crate) fn new(nombre: String, apellido: String) -> Self {
        Self { nombre, apellido }
    }
}
