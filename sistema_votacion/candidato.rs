use ink::primitives::AccountId;

use crate::enums::EstadoAprobacion;

#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
#[derive(Debug)]
/// Representa un candidato de una eleccion determinada.
/// Almacena su AccountId, estado de aprobacion y cantidad de votos recibidos.
pub struct Candidato {
    pub id: AccountId,
    pub(crate) aprobacion: EstadoAprobacion,
    votos: u32,
}

impl Candidato {
    /// Construye un nuevo candidato con el AccountId dado.
    /// Ademas tiene estado de aprobacion pendiente y cero votos recibidos.
    pub(crate) fn new(id: AccountId) -> Self {
        Self {
            id,
            aprobacion: EstadoAprobacion::Pendiente,
            votos: 0,
        }
    }

    /// Retorna `true` si el candidato está aprobado, sino `false`
    pub fn esta_aprobado(&self) -> bool {
        matches!(self.aprobacion, EstadoAprobacion::Aprobado)
    }

    /// Retorna `true` si el candidato está rechazado, sino `false`
    pub fn esta_rechazado(&self) -> bool {
        matches!(self.aprobacion, EstadoAprobacion::Rechazado)
    }

    /// Retorna `true` si el candidato está en estado pendiente, sino `false`
    pub fn esta_pendiente(&self) -> bool {
        matches!(self.aprobacion, EstadoAprobacion::Pendiente)
    }
}
