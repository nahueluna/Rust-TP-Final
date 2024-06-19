use crate::enums::EstadoAprobacion;
use crate::votante::Votante;
use crate::{candidato::Candidato, fecha::Fecha};
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

    /// Busca un votante o un candidado con un AccountId determinado.
    /// Si lo encuentra retorna Some<indice> sino None.
    pub fn buscar_miembro(&self, id: &AccountId, rol: Option<&Rol>) -> Option<usize> {
        match rol {
            Some(r) => match r {
                Rol::Candidato => self.candidatos.iter().position(|v| v.id == *id),
                Rol::Votante => self.votantes.iter().position(|v| v.id == *id),
            },
            None => {
                self.candidatos.iter().position(|v| v.id == *id).
                or(self.votantes.iter().position(|v| v.id == *id))
            }
        }
    }

    /// Retorna si un miembro especificando el rol fue aprobado
    /// Usa unwrap no usar con un id no existente
    pub(crate) fn esta_aprobado(&self, id_miembro: &AccountId, rol: &Rol) -> bool {
        match rol {
            Rol::Candidato => self.candidatos.iter().find(|v| &v.id == id_miembro).unwrap().está_aprobado(),
            Rol::Votante => self.votantes.iter().find(|v| &v.id == id_miembro).unwrap().está_aprobado(),
        }
    }

    /// Retorna un `Vec<AccountId>` de los usuarios que se correspondan al rol `rol`.
    pub fn get_no_verificados(&self, rol: Rol) -> Vec<AccountId> {
        let mut no_verificados: Vec<AccountId> = Vec::new();

        match rol {
            Rol::Votante => {
                for v in self.votantes.iter() {
                    if !v.está_aprobado() {
                        no_verificados.push(v.id);
                    }
                }
            }
            Rol::Candidato => {
                for c in self.candidatos.iter() {
                    if !c.está_aprobado() {
                        no_verificados.push(c.id);
                    }
                }
            }
        }

        no_verificados
    }

    /// Aprueba un miembro especificando el rol
    pub(crate) fn aprobar(&mut self, id_miembro: AccountId, rol: &Rol) {
        match rol {
            Rol::Votante => {
                if let Some(v) = self.votantes.iter_mut().find(|v| v.id == id_miembro) {
                    v.aprobacion = EstadoAprobacion::Aprobado;
                }
            }
            Rol::Candidato => {
                if let Some(v) = self.candidatos.iter_mut().find(|v| v.id == id_miembro) {
                    v.aprobacion = EstadoAprobacion::Aprobado;
                }
            }
        }
    }
}
