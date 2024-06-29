use ink::primitives::AccountId;

use crate::eleccion::Miembro;
use crate::enums::Error;

#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
#[derive(Debug)]
/// Representa un votante en una eleccion determinada.
/// Almacena su AccountId, estado de aprobacion y si voto o no.
pub struct Votante {
    pub(crate) id: AccountId,
    pub(crate) ha_votado: bool,
}

//#[ink::trait_definition]
impl Miembro for Votante {
    /// Si el votante no `ha_votado`, se invierte el booleano `ha_votado`.
    /// Si el votante `ha_votado` se devuelve un `Error::VotanteYaVoto`
    fn votar(&mut self) -> Result<(), Error> {
        if self.ha_votado {
            Err(Error::VotanteYaVoto)
        } else {
            self.ha_votado = !self.ha_votado;
            Ok(())
        }
    }
}

impl Votante {
    /// Construye un nuevo votante con el AccountId.
    /// Ademas tiene estado de aprobacion pendiente y no ha votado.
    pub fn new(id: AccountId) -> Self {
        Self {
            id,
            ha_votado: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn probar_creacion() {
        let votante_id: [u8; 32] = [0; 32];
        let votante = Votante::new(AccountId::from(votante_id));
        assert!(!votante.ha_votado);

        let votante_id: [u8; 32] = [255; 32];
        let votante = Votante::new(AccountId::from(votante_id));
        assert!(!votante.ha_votado);
    }

    #[test]
    fn probar_votar() {
        let votante_id: [u8; 32] = [0; 32];
        let mut votante = Votante::new(AccountId::from(votante_id));
        assert!(votante.votar().is_ok());
        assert!(votante.ha_votado);
        assert!(votante.votar().is_err());
    }
}
