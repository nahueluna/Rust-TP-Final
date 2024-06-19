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
    id: u32,
    pub votantes: Vec<Votante>,
    candidatos: Vec<Candidato>,
    puesto: String,
    inicio: Fecha,
    fin: Fecha,
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

    ///Busca un votante con un AccountId determinado.
    ///Si lo encuentra retorna Some<indice> sino None.
    pub fn buscar_votante(&self, id: AccountId) -> Option<usize> {
        self.votantes.iter().position(|v| v.id == id)
    }

    /// Retorna un `Vec<AccountId>` de tanto los votantes como los candidatos que aún no están verificados.
    pub fn get_no_verificados(&self) -> Vec<AccountId> {
        let mut no_verificados: Vec<AccountId> = Vec::new();
        for v in self.votantes.iter() {
            if !v.está_aprobado() {
                no_verificados.push(v.id);
            }
        }
        for c in self.candidatos.iter() {
            if !c.está_aprobado() {
                no_verificados.push(c.id);
            }
        }
        no_verificados
    }
}
