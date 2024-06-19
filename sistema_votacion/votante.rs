use crate::enums::EstadoAprobacion;

#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
#[derive(Debug)]
/// Representa un votante en una eleccion determinada.
/// Almacena su dni, estado de aprobacion y si voto o no.
pub struct Votante {
    dni: u32,
    aprobacion: EstadoAprobacion,
    ha_votado: bool,
}

impl Votante {
    /// Construye un nuevo votante con el dni dado.
    /// Ademas tiene estado de aprobacion pendiente y no ha votado.
    pub(crate) fn new(dni: u32) -> Self {
        Self {
            dni,
            aprobacion: EstadoAprobacion::Pendiente,
            ha_votado: false,
        }
    }
}
