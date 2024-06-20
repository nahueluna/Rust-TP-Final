use ink::primitives::AccountId;

use crate::{eleccion::Miembro, enums::{Error, EstadoAprobacion}};

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

impl Miembro for Candidato {
    /// Retorna `true` si el candidato está aprobado, sino `false`
    fn esta_aprobado(&self) -> bool {
        matches!(self.aprobacion, EstadoAprobacion::Aprobado)
    }

    /// Retorna `true` si el candidato está rechazado, sino `false`
    fn esta_rechazado(&self) -> bool {
        matches!(self.aprobacion, EstadoAprobacion::Rechazado)
    }

    /// Retorna `true` si el candidato está en estado pendiente, sino `false`
    fn esta_pendiente(&self) -> bool {
        matches!(self.aprobacion, EstadoAprobacion::Pendiente)
    }

    // Incrementa en uno la cantidad de votos
    fn votar(&mut self) -> Result<(), Error> {
        self.votos += 1;
        Ok(())
    }

    /// Modifica el estado de aprobacion si el recibido es distinto de `Pendiente`
    fn cambiar_estado_aprobacion(&mut self, estado: EstadoAprobacion) {
        match estado {
            EstadoAprobacion::Pendiente => (),
            _ => self.aprobacion = estado,
        }
    }
}

impl Candidato {
    /// Construye un nuevo candidato con el AccountId dado.
    /// Ademas tiene estado de aprobacion pendiente y cero votos recibidos.
    pub fn new(id: AccountId) -> Self {
        Self {
            id,
            aprobacion: EstadoAprobacion::Pendiente,
            votos: 0,
        }
    }
}
