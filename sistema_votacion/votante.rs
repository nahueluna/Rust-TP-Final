use ink::primitives::AccountId;

use crate::enums::EstadoAprobacion;

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

impl Votante {
    /// Construye un nuevo votante con el AccountId.
    /// Ademas tiene estado de aprobacion pendiente y no ha votado.
    pub(crate) fn new(id: AccountId) -> Self {
        Self {
            id,
            aprobacion: EstadoAprobacion::Pendiente,
            ha_votado: false,
        }
    }

    /// Retorna `true` si el votante está aprobado, sino `false`
    pub fn esta_aprobado(&self) -> bool {
        matches!(self.aprobacion, EstadoAprobacion::Aprobado)
    }

    /// Retorna `true` si el votante está rechazado, sino `false`
    pub fn esta_rechazado(&self) -> bool {
        matches!(self.aprobacion, EstadoAprobacion::Rechazado)
    }

    /// Retorna `true` si el votante está en estado pendiente, sino `false`
    pub fn esta_pendiente(&self) -> bool {
        matches!(self.aprobacion, EstadoAprobacion::Pendiente)
    }

    /// Modifica el estado de aprobacion si el recibido es distinto de `Pendiente`
    pub fn cambiar_estado_aprobacion(&mut self, estado: EstadoAprobacion) {
        match estado {
            EstadoAprobacion::Pendiente => (),
            _ => self.aprobacion = estado,
        }      
    }
    
}