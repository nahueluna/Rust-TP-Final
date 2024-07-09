use ink::primitives::AccountId;

use crate::{eleccion::Miembro, enums::Error};

/// Representa un candidato de una eleccion determinada.
/// Almacena su `AccountId` y cantidad de votos recibidos.
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
#[derive(Debug, PartialEq, Eq)]
pub struct Candidato {
    id: AccountId,
    votos: u32,
}

impl Miembro for Candidato {
    /// Incrementa en uno la cantidad de votos recibidos
    fn votar(&mut self) -> Result<(), Error> {
        self.votos += 1;
        Ok(())
    }

    fn get_account_id(&self) -> AccountId {
        self.id
    }

    fn get_votos(&self) -> u32 {
        self.votos
    }
}

impl Candidato {
    /// Construye un nuevo candidato con el `AccountId` dado.
    /// Inicializa con cero votos recibidos.
    pub fn new(id: AccountId) -> Self {
        Self { id, votos: 0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn probar_creacion_candidato() {
        let candidato_id: [u8; 32] = [5; 32];
        let candidato = Candidato::new(AccountId::from(candidato_id));
        assert_eq!(candidato.votos, 0);
    }

    #[test]
    fn probar_votar_candidato() {
        let candidato_id: [u8; 32] = [5; 32];
        let mut candidato = Candidato::new(AccountId::from(candidato_id));
        assert!(candidato.votar().is_ok());
        assert_eq!(candidato.votos, 1);
    }
}
