use windows::{
    Win32::{
        Foundation::{FARPROC, HMODULE},
        System::LibraryLoader::GetProcAddress,
    },
    core::s,
};

static mut DINPUT: DInput = DInput::new();

struct DInput {
    DirectInput8Create: FARPROC,
    DllCanUnloadNow: FARPROC,
    DllGetClassObject: FARPROC,
    DllRegisterServer: FARPROC,
    DllUnregisterServer: FARPROC,
    GetdfDIJoystick: FARPROC,
}

impl DInput {
    const fn new() -> Self {
        Self {
            DirectInput8Create: FARPROC::None,
            DllCanUnloadNow: FARPROC::None,
            DllGetClassObject: FARPROC::None,
            DllRegisterServer: FARPROC::None,
            DllUnregisterServer: FARPROC::None,
            GetdfDIJoystick: FARPROC::None,
        }
    }

    fn init(&mut self, dll: HMODULE) {
        unsafe {
            self.DirectInput8Create = GetProcAddress(dll, s!("DirectInput8Create"));
            self.DllCanUnloadNow = GetProcAddress(dll, s!("DllCanUnloadNow"));
            self.DllGetClassObject = GetProcAddress(dll, s!("DllGetClassObject"));
            self.DllRegisterServer = GetProcAddress(dll, s!("DllRegisterServer"));
            self.DllUnregisterServer = GetProcAddress(dll, s!("DllUnregisterServer"));
            self.GetdfDIJoystick = GetProcAddress(dll, s!("GetdfDIJoystick"));
        }
    }
}

#[unsafe(no_mangle)]
fn DirectInput8Create() {
    unsafe { DINPUT.DirectInput8Create.unwrap()() };
}
#[unsafe(no_mangle)]
fn DllCanUnloadNow() {
    unsafe { DINPUT.DllCanUnloadNow.unwrap()() };
}
#[unsafe(no_mangle)]
fn DllGetClassObject() {
    unsafe { DINPUT.DllGetClassObject.unwrap()() };
}
#[unsafe(no_mangle)]
fn DllRegisterServer() {
    unsafe { DINPUT.DllRegisterServer.unwrap()() };
}
#[unsafe(no_mangle)]
fn DllUnregisterServer() {
    unsafe { DINPUT.DllUnregisterServer.unwrap()() };
}
#[unsafe(no_mangle)]
fn GetdfDIJoystick() {
    unsafe { DINPUT.GetdfDIJoystick.unwrap()() };
}

#[allow(static_mut_refs)]
pub fn init(dll: HMODULE) {
    unsafe { DINPUT.init(dll) };
}
