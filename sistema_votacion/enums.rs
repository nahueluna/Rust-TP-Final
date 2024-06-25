#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
#[derive(Debug, PartialEq)]
/// Representa el estado de aprobacion de un usuario.
///
/// Los usuarios registrados como votantes o candidatos,
/// inician en estado de aprobacion pendiente para una eleccion determinada.
/// Luego el administrador puede aprobarlos o rechazarlos.
pub enum EstadoAprobacion {
    Pendiente,
    Aprobado,
    Rechazado,
}

#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
#[derive(Debug)]
/// Representa un error al llamar a un metodo del sistema.
pub enum Error {
    PermisosInsuficientes,  // Intentar acceder a un metodo del administrador sin serlo.
    UsuarioExistente,       // Intentar registrar un usuario que ya existe.
    UsuarioNoExistente,     // Intentar registrar como votante/candidato a un usuario que no existe.
    VotanteExistente,       // Intentar registrar un votante que ya existe.
    CandidatoExistente,     // Intentar registrar un candidato que ya existe.
    VotanteNoExistente,     // Intentar aprobar un votante que no existe.
    CandidatoNoExistente,   // Intentar aprobar un candidato que no existe.
    VotanteYaAprobado,      // Intentar aprobar un votante que ya fue aprpbado.
    CandidatoYaAprobado,    // Intentar aprobar un candidato que ya fue aprobado.
    VotanteYaRechazado,     // Intentar rechazar un candidato que ya fue rechazado
    CandidatoYaRechazado,   // Intentar rechazar un candidato que ya fue rechazado
    VotacionNoExiste,       // Intentar registrar un votante en una eleccion que no existe.
    VotacionNoIniciada,     // Intenta obtener los candidatos disponibles en una eleccion que no esta en curso
    VotacionFinalizada,     // La votación finalizó, no es posible operar
    VotanteYaVoto,          // El votante ya votó, no puede hacerlo dos veces
    FechaInvalida           // Se intenta crear una elección donde la fecha fin > inicio
}

#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
#[derive(Debug, PartialEq)]
// Estados que puede tener la eleccion
pub enum EstadoDeEleccion {
    Pendiente,
    EnCurso,
    Finalizada,
}

