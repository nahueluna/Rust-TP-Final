use crate::enums::{Error, EstadoAprobacion, EstadoDeEleccion};
use crate::votante::Votante;
use crate::{candidato::Candidato, fecha::Fecha};
use ink::prelude::string::ToString;
use ink::prelude::{string::String, vec::Vec};
use ink::primitives::AccountId;

/*
 * Eleccion: identificador, fechas de inicio y cierre.
 * Votantes con su id propio, estado de aprobacion, y si votaron o no.
 * Candidatos con id propio, estado de aprobacion, y cantidad de votos recibidos.
 */
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
#[derive(Debug)]
pub(crate) struct Eleccion {
    pub(crate) id: u32,
    votantes: Vec<Votante>,
    candidatos: Vec<Candidato>,
    puesto: String,
    pub inicio: Fecha,
    pub fin: Fecha,
    estado: EstadoDeEleccion,
}

#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub enum Rol {
    Candidato,
    Votante,
}

pub trait Miembro {
    fn esta_aprobado(&self) -> bool;
    fn esta_rechazado(&self) -> bool;
    fn esta_pendiente(&self) -> bool;
    fn votar(&mut self) -> Result<(), Error>;
    fn cambiar_estado_aprobacion(&mut self, estado: EstadoAprobacion);
}

impl Eleccion {
    // Creacion de una eleccion vacia
    pub(crate) fn new(id: u32, puesto: String, inicio: Fecha, fin: Fecha) -> Self {
        Self {
            id,
            votantes: Vec::new(),
            candidatos: Vec::new(),
            puesto,
            inicio,
            fin,
            estado: EstadoDeEleccion::Pendiente,
        }
    }

    pub(crate) fn añadir_miembro(&mut self, id: AccountId, rol: Rol) {
        match rol {
            Rol::Candidato => {
                self.candidatos.push(Candidato::new(id));
            }
            Rol::Votante => {
                self.votantes.push(Votante::new(id));
            }
        }
    }

    /// Devuelve `Some(index)` del usuario con el id y rol especificado o
    /// `None` en caso de no encontrarlo.
    fn get_posicion_miembro(&self, id: &AccountId, rol: &Rol) -> Option<usize> {
        match rol {
            Rol::Candidato => self.candidatos.iter().position(|v| v.id == *id),
            Rol::Votante => self.votantes.iter().position(|v| v.id == *id),
        }
    }

    /// Busca un votante o un candidato con un AccountId determinado
    /// Retorna `Some(&mut Votante)` o `Some(&mut Candidato)`, respectivamente,
    /// si lo halla, sino devuelve `None`.
    pub fn buscar_miembro(&mut self, id: &AccountId, rol: &Rol) -> Option<&mut dyn Miembro> {
        match rol {
            Rol::Candidato => {
                if let Some(c) = self.candidatos.iter_mut().find(|v| v.id == *id) {
                    Some(c)
                } else {
                    None
                }
            }
            Rol::Votante => {
                if let Some(v) = self.votantes.iter_mut().find(|v| v.id == *id) {
                    Some(v)
                } else {
                    None
                }
            }
        }
    }

    /// Retorna si el usuario con `AccoundId` especificado existe en la eleccion,
    /// sea `Candidato` o `Votante`
    pub fn existe_usuario(&self, id: &AccountId) -> bool {
        self.votantes.iter().any(|vot| vot.id == *id)
            || self.candidatos.iter().any(|cand| cand.id == *id)
    }

    /// Retorna un `Vec<AccountId>` de los usuarios que se correspondan al rol `rol`.
    pub fn get_no_verificados(&self, rol: Rol) -> Vec<AccountId> {
        let mut no_verificados: Vec<AccountId> = Vec::new();

        match rol {
            Rol::Votante => {
                for v in self.votantes.iter() {
                    if v.esta_pendiente() {
                        no_verificados.push(v.id);
                    }
                }
            }
            Rol::Candidato => {
                for c in self.candidatos.iter() {
                    if c.esta_pendiente() {
                        no_verificados.push(c.id);
                    }
                }
            }
        }

        no_verificados
    }

    /// Retorna una lista de candidatos aprobados. Si no los hay retorna la lista vacía.
    pub fn get_candidatos(&self) -> Vec<AccountId> {
        self.candidatos
            .iter()
            .filter(|c| c.esta_aprobado())
            .map(|c| c.id)
            .collect()
    }

    // Permite que el votante `id_votante` vote al candidato `id_cantidato`
    // Una vez que esto ocurre, el votante no puede volver a votar
    pub fn votar(&mut self, id_votante: AccountId, id_candidato: AccountId) -> Result<(), Error> {
        // El código está raro con el fin no romper las reglas de ownership
        if self.estado != EstadoDeEleccion::EnCurso {
            Err(Error::VotacionFueraDeTermino)
        } else if self
            .buscar_miembro(&id_candidato, &Rol::Candidato)
            .is_none()
        {
            Err(Error::CandidatoNoExistente)
        } else if let Some(votante) = self.buscar_miembro(&id_votante, &Rol::Votante) {
            votante.votar().map(|()| {
                self.buscar_miembro(&id_candidato, &Rol::Votante)
                    .unwrap()
                    .votar()
            })?
        } else {
            Err(Error::VotanteNoExistente)
        }
    }
}
