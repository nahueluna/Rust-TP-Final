use crate::enums::{EstadoAprobacion, Error};
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

    /// Retorna `true` un miembro especificando el rol fue aprobado.
    /// Retorna `false` si el miembro no fue aprobado o no existe
    pub(crate) fn esta_aprobado(&self, id_miembro: &AccountId, rol: &Rol) -> bool {
        match rol {
            Rol::Candidato => {
                if let Some(c) = self.candidatos.iter().find(|v| &v.id == id_miembro) {
                    return c.esta_aprobado();
                }
            }
            Rol::Votante => {
                if let Some(v) = self.votantes.iter().find(|v| &v.id == id_miembro) {
                    return v.esta_aprobado();
                }
            }
        }

        false
    }

    /// Retorna `true` un miembro especificando el rol fue rechazado.
    /// Retorna `false` si el miembro no fue aprobado o no existe
    pub(crate) fn esta_rechazado(&self, id_miembro: &AccountId, rol: &Rol) -> bool {
        match rol {
            Rol::Candidato => {
                if let Some(c) = self.candidatos.iter().find(|v| &v.id == id_miembro) {
                    return c.esta_rechazado();
                }
            }
            Rol::Votante => {
                if let Some(v) = self.votantes.iter().find(|v| &v.id == id_miembro) {
                    return v.esta_rechazado();
                }
            }
        }

        false
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

    /// Aprueba un miembro especificando el rol.
    /// Retorna `Ok()` si fue posible o `Error` si el usuario no se encontró
    pub(crate) fn aprobar(&mut self, id_miembro: AccountId, rol: &Rol) -> Result<(), Error> {
        match rol {
            Rol::Votante => {
                if let Some(v) = self.votantes.iter_mut().find(|v| v.id == id_miembro) {
                   v.aprobacion = EstadoAprobacion::Aprobado;
                   return Ok(());
                }
                Err(Error::CandidatoNoExistente)
            }
            Rol::Candidato => {
                if let Some(v) = self.candidatos.iter_mut().find(|v| v.id == id_miembro) {
                    v.aprobacion = EstadoAprobacion::Aprobado;
                    return Ok(());
                }
                Err(Error::VotanteNoExistente)
            }
        }
    }

    /// Rechaza un miembro especificando el rol.
    /// Retorna `Ok()` si fue posible o `Error` si el usuario no se encontró
    pub(crate) fn rechazar(&mut self, id_miembro: AccountId, rol: &Rol) -> Result<(), Error> {
        match rol {
            Rol::Votante => {
                if let Some(v) = self.votantes.iter_mut().find(|v| v.id == id_miembro) {
                   v.aprobacion = EstadoAprobacion::Rechazado;
                   return Ok(());
                }
                Err(Error::CandidatoNoExistente)
            }
            Rol::Candidato => {
                if let Some(v) = self.candidatos.iter_mut().find(|v| v.id == id_miembro) {
                    v.aprobacion = EstadoAprobacion::Rechazado;
                    return Ok(());
                }
                Err(Error::VotanteNoExistente)
            }
        }
    }
}
