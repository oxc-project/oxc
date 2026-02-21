/// Runtime diagnostic constants.
///
/// Port of `Utils/RuntimeDiagnosticConstants.ts` from the React Compiler.
///
/// Constants synced with the react-compiler-runtime GuardKind enum.
/// Guard kinds for hook runtime diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum GuardKind {
    PushHookGuard = 0,
    PopHookGuard = 1,
    AllowHook = 2,
    DisallowHook = 3,
}
