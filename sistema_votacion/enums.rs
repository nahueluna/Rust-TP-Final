use core::fmt::Display;

#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
#[derive(Debug, PartialEq)]
/// Representa el estado de aprobacion de un usuario.
///
/// Los usuarios registrados como votantes o candidatos,
/// inician en estado de aprobacion pendiente para una eleccion determinada.
/// Luego el administrador puede aprobarlos o rechazarlos.
pub enum EstadoAprobacion {
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
    VotacionNoExiste,       // Intentar registrar un votante en una eleccion que no existe.
    VotacionNoIniciada,     // Intenta obtener los candidatos disponibles en una eleccion que no esta en curso
    VotacionFinalizada,     // La votación finalizó, no es posible operar
    VotanteYaVoto,          // El votante ya votó, no puede hacerlo dos veces
    FechaInvalida           // Se intenta crear una elección donde la fecha fin > inicio
}

impl Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::PermisosInsuficientes => write!(f, "El usuario no posee los permisos de administracion requeridos"),
            Error::UsuarioExistente => write!(f, "El usuario ya se encuentra registrado en el sistema"),
            Error::UsuarioNoExistente => write!(f, "El usuario no se encuentra registrado en el sistema"),
            Error::VotanteExistente => write!(f, "El votante ya se encuentra registrado"),
            Error::CandidatoExistente => write!(f, "El candidato ya se encuentra registrado"),
            Error::VotanteNoExistente => write!(f, "El votante solicitado no se encuentra registrado"),
            Error::CandidatoNoExistente => write!(f, "El candidato solicitado no se encuentra registrado"),
            Error::VotacionNoExiste => write!(f, "La votación solicitada no existe en el sistema"),
            Error::VotacionNoIniciada => write!(f, "La votación solicitada no ha comenzado"),
            Error::VotacionFinalizada => write!(f, "La votación solicitada ya ha finalizado"),
            Error::VotanteYaVoto => write!(f, "El votante solicitado ya ha votado"),
            Error::FechaInvalida => write!(f, "La fecha de finalizacion ingresada no es consistente con la de inicio"),
        }
    }
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

