use ink::primitives::AccountId;

use crate::eleccion::Miembro;
use crate::enums::{Error, EstadoAprobacion};

#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
#[derive(Debug)]
/// Representa un votante en una eleccion determinada.
/// Almacena su AccountId, estado de aprobacion y si voto o no.
pub struct Votante {
    pub id: AccountId,
    pub(crate) aprobacion: EstadoAprobacion,
    ha_votado: bool,
}

//#[ink::trait_definition]
impl Miembro for Votante {
    /// Retorna `true` si el votante está aprobado, sino `false`
    fn esta_aprobado(&self) -> bool {
        matches!(self.aprobacion, EstadoAprobacion::Aprobado)
    }

    /// Retorna `true` si el votante está rechazado, sino `false`
    fn esta_rechazado(&self) -> bool {
        matches!(self.aprobacion, EstadoAprobacion::Rechazado)
    }

    /// Retorna `true` si el votante está en estado pendiente, sino `false`
    fn esta_pendiente(&self) -> bool {
        matches!(self.aprobacion, EstadoAprobacion::Pendiente)
    }

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

    /// Modifica el estado de aprobacion si el recibido es distinto de `Pendiente`
    fn cambiar_estado_aprobacion(&mut self, estado: EstadoAprobacion) {
        match estado {
            EstadoAprobacion::Pendiente => (),
            _ => self.aprobacion = estado,
        }
    }
}

impl Votante {
    /// Construye un nuevo votante con el AccountId.
    /// Ademas tiene estado de aprobacion pendiente y no ha votado.
    pub fn new(id: AccountId) -> Self {
        Self {
            id,
            aprobacion: EstadoAprobacion::Pendiente,
            ha_votado: false,
        }
    }
}
