use core::fmt::Display;

/// Representa el estado de aprobacion de un usuario.
///
/// Utilizado para decidir el estado de aprobación de un usuario
/// en el proceso de cambio de estado
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
#[derive(Debug, PartialEq, Clone)]
pub enum EstadoAprobacion {
    Aprobado,
    Rechazado,
}

#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
#[derive(Debug, PartialEq)]
/// Estados que puede tener la eleccion según su fecha de inicio y cierre
pub enum EstadoDeEleccion {
    Pendiente,
    EnCurso,
    Finalizada,
}

/// Representa un error al llamar a un metodo del sistema.
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
#[derive(Debug,PartialEq)]
pub enum Error {
    PermisosInsuficientes,      // Intentar acceder a un metodo que no le corresponde al usuario.
    UsuarioExistente,           // Intentar registrar un usuario que ya existe.
    UsuarioNoExistente,         // Intentar registrar como votante/candidato a un usuario que no existe.
    UsuarioNoPermitido,         // Intentar registrar administrador como usuario del sistema (futuro candidato o miembro)
    VotanteExistente,           // Intentar registrar un votante que ya existe.
    CandidatoExistente,         // Intentar registrar un candidato que ya existe.
    MiembroExistente,           // Intentar registrar un miembro que ya existe.
    VotanteNoExistente,         // Intentar aprobar un votante que no existe.
    CandidatoNoExistente,       // Intentar aprobar un candidato que no existe.
    VotacionNoExiste,           // Intentar registrar un votante en una eleccion que no existe.
    VotacionNoIniciada,         // Intenta obtener los candidatos disponibles en una eleccion que no esta en curso
    VotacionEnCurso,            // Intentar registrar a un miembro en una eleccion que ya inicio.
    VotacionFinalizada,         // La votación finalizó, no es posible operar
    VotanteYaVoto,              // El votante ya votó, no puede hacerlo dos veces
    FechaFinalizacionInvalida,  // Se intenta crear una elección donde la fecha fin > inicio
    FechaInvalida               // La fecha introducida no existe (no es valida)
}

impl Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::PermisosInsuficientes => write!(
                f,
                "El usuario no posee los permisos requeridos"
            ),
            Error::UsuarioExistente => {
                write!(f, "El usuario ya se encuentra registrado en el sistema")
            }
            Error::UsuarioNoExistente => {
                write!(f, "El usuario no se encuentra registrado en el sistema")
            }
            Error::UsuarioNoPermitido => write!(
                f,
                "El usuario que intenta registrar es el administrador del sistema"
            ),
            Error::VotanteExistente => write!(f, "El votante ya se encuentra registrado"),
            Error::CandidatoExistente => write!(f, "El candidato ya se encuentra registrado"),
            Error::MiembroExistente => write!(f, "El miembro ya se encuentra registrado"),
            Error::VotanteNoExistente => {
                write!(f, "El votante solicitado no se encuentra registrado")
            }
            Error::CandidatoNoExistente => {
                write!(f, "El candidato solicitado no se encuentra registrado")
            }
            Error::VotacionNoExiste => write!(f, "La votación solicitada no existe en el sistema"),
            Error::VotacionNoIniciada => write!(f, "La votación solicitada no ha comenzado"),
            Error::VotacionEnCurso => write!(f, "La votación se encuentra en curso."),
            Error::VotacionFinalizada => write!(f, "La votación solicitada ya ha finalizado"),
            Error::VotanteYaVoto => write!(f, "El votante solicitado ya ha votado"),
            Error::FechaFinalizacionInvalida => write!(
                f,
                "La fecha de finalizacion ingresada no es consistente con la de inicio"
            ),
            Error::FechaInvalida => write!(f, "La fecha ingresada no es valida"),
        }
    }
}