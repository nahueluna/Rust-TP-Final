use ink::primitives::AccountId;

use crate::eleccion::Miembro;
use crate::enums::Error;

#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
#[derive(Debug, PartialEq)]
/// Representa un votante en una eleccion determinada.
/// Almacena su `AccountId` y si voto o no.
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

    fn get_account_id(&self) -> AccountId {
        self.id
    }

    fn get_votos(&self) -> u32 {
        if self.ha_votado {
            1
        } else {
            0
        }
    }
}

impl Votante {
    /// Construye un nuevo votante con el `AccountId`.
    /// Inicializa con `ha_votado` en `false`
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

    #[test]
    fn probar_get_votos() {
        // Creo un votante
        let votante_id: [u8; 32] = [0; 32];
        let mut votante = Votante::new(AccountId::from(votante_id));
        
        assert_eq!(votante.get_votos(),0); // Como no voto, get_votos() tiene que devolver 0 
        votante.votar().unwrap();
        assert_eq!(votante.get_votos(),1); // Como voto, get_votos() tiene que devolver 1 
    }
}
