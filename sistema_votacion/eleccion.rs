use crate::enums::EstadoDeEleccion;
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
    inicio: Fecha,
    fin: Fecha,
    estado: EstadoDeEleccion,
}

#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub enum Rol {
    Candidato,
    Votante,
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

    /// Busca un votante con un AccountId determinado
    /// Retorna `Some(&mut Votante)` si lo halla, sino devuelve `None`.
    pub fn buscar_votante(&mut self, id: &AccountId) -> Option<&mut Votante> {
        if let Some(index) = self.get_posicion_miembro(id, &Rol::Votante) {
            return self.votantes.get_mut(index);
        }

        None
    }

    /// Busca un candidato con un AccountId determinado
    /// Retorna `Some(&mut Candidato)` si lo halla, sino devuelve `None`.
    pub fn buscar_candidato(&mut self, id: &AccountId) -> Option<&mut Candidato> {
        if let Some(index) = self.get_posicion_miembro(id, &Rol::Candidato) {
            return self.candidatos.get_mut(index);
        }

        None
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

    /// Retorna una lista de candidatos. Si no los hay retorna la lista vacía.
    pub fn get_candidatos(&self) -> Vec<AccountId> {
        self.candidatos.iter().map(|c| c.id).collect()
    }

    pub fn consultar_estado(&self) -> String {
        match self.estado {
            EstadoDeEleccion::EnCurso => "La eleccion está en curso".to_string(),
            EstadoDeEleccion::Pendiente => "La eleccion todavia no inició".to_string(),
            EstadoDeEleccion::Finalizada => "La eleccion ya ha finalizado".to_string(),
        }
    }
}
