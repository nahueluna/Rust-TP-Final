use ink::primitives::AccountId;

use crate::enums::EstadoAprobacion;

#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
#[derive(Debug)]
/// Representa un votante en una eleccion determinada.
/// Almacena su AccountId, estado de aprobacion y si voto o no.
pub struct Votante {
    id: AccountId,
    aprobacion: EstadoAprobacion,
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
}
