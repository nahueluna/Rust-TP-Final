use crate::fecha::Fecha;
use crate::votante::Votante;
use ink::prelude::{string::String, vec::Vec};

/*
 * Eleccion: identificador, fechas de inicio y cierre.
 * Votantes con su id propio, estado de aprobacion, y si votaron o no.
 * Candidatos con id propio y cantidad de votos recibidos (preferible que sea HashMap)
 */
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
#[derive(Debug)]
pub(crate) struct Eleccion {
    id: u32,
    votantes: Vec<Votante>,
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
