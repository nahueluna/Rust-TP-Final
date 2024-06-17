use ink::prelude::string::String;

/*
 * Administrador electoral. Se encarga de crear las elecciones y configurar todos sus apartados.
 */
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
#[derive(Debug)]
pub(crate) struct Administrador {
    hash: String,
    nombre: String,
    apellido: String,
    dni: u32,
}

impl Administrador {
    // Creacion del administrador con toda la informacion requerida
    pub(crate) fn new(hash: String, nombre: String, apellido: String, dni: u32) -> Self {
        Self {
            hash,
            nombre,
            apellido,
            dni,
        }
    }
}
