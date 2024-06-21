use ink::primitives::AccountId;
use ink::prelude::string::String;

#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[derive(Debug)]
pub struct ReporteVotantes {
    id: AccountId,
    nombre: String,
    apellido: String,
}

impl ReporteVotantes {
    pub fn new(id: AccountId, nombre: String, apellido: String) -> Self {
        Self {
            id,
            nombre,
            apellido,
        }
    }
}
