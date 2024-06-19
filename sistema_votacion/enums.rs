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
    PermisosInsuficientes, //Intentar acceder a un metodo del administrador sin serlo.
    UsuarioExistente, //Intentar registrar un usuario que ya existe.
}
