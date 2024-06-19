#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
#[derive(Debug)]
/// Representa el estado de aprobacion de un usuario.
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
    PermisosInsuficientes,   //Intentar acceder a un metodo del administrador sin serlo.
    UsuarioExistente,        //Intentar registrar un usuario que ya existe.
    UsuarioNoExistente,      //Intentar registrar como votante/candidato a un usuario que no existe.
    VotanteExistente,        //Intentar registrar un votante que ya existe.
    CandidatoExistente,      //Intentar registrar un candidato que ya existe.
    VotanteNoExistente,      //Intentar aprobar un votante que no existe.
    CandidatoNoExistente,    //Intentar aprobar un candidato que no existe.
    VotanteYaAprobado,       //Intentar aprobar un votante que ya fue aprpbado.
    CandidatoYaAprobado,     //Intentar aprobar un candidato que ya fue aprpbado.
    VotacionNoExiste,        //Intentar registrar un votante en una eleccion que no existe.
}
