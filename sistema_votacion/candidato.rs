use crate::enums::EstadoAprobacion;

#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
#[derive(Debug)]
/// Representa un candidato de una eleccion determinada.
/// Almacena su dni, estado de aprobacion y cantidad de votos recibidos.
pub struct Candidato {
    dni: u32,
    aprobacion: EstadoAprobacion,
    votos: u32,
}

impl Candidato {
    /// Construye un nuevo candidato con el dni dado.
    /// Ademas tiene estado de aprobacion pendiente y cero votos recibidos.
    pub(crate) fn new(dni: u32) -> Self {
        Self {
            dni,
            aprobacion: EstadoAprobacion::Pendiente,
            votos: 0,
        }
    }
}
