use ink::prelude::string::String;
use ink::primitives::AccountId;

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

#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[derive(Debug)]
pub struct ReporteParticipacion {
    votaron: u64,
    total_votantes: u64,
}

impl ReporteParticipacion {
    pub fn new(votaron: u64, total_votantes: u64) -> Self {
        Self {
            votaron,
            total_votantes,
        }
    }
}
