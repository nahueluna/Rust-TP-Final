use ink::primitives::AccountId;

use crate::{eleccion::Miembro, enums::Error};

#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
#[derive(Debug)]
/// Representa un candidato de una eleccion determinada.
/// Almacena su AccountId, estado de aprobacion y cantidad de votos recibidos.
pub struct Candidato {
    pub id: AccountId,
    votos: u32,
}

impl Miembro for Candidato {
    /// Incrementa en uno la cantidad de votos
    fn votar(&mut self) -> Result<(), Error> {
        self.votos += 1;
        Ok(())
    }
}

impl Candidato {
    /// Construye un nuevo candidato con el AccountId dado.
    /// Ademas tiene estado de aprobacion pendiente y cero votos recibidos.
    pub fn new(id: AccountId) -> Self {
        Self {
            id,
            votos: 0,
        }
    }
}
