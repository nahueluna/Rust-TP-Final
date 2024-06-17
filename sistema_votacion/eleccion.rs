use crate::fecha::Fecha;
use ink::prelude::{string::String, vec::Vec};

/*
 * Eleccion: identificador, fechas de inicio y cierre.
 * Votantes con id propio y del candidato votado.
 * Candidatos con id propio y cantidad de votos recibidos (preferible que sea HashMap)
 */
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
#[derive(Debug)]
pub(crate) struct Eleccion {
    id: u32,
    votantes: Vec<(u32, Option<u32>)>,
    candidatos: Vec<(u32, u16)>,
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
}
