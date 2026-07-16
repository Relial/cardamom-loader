use windows::{
    Win32::{
        Foundation::{FARPROC, HMODULE},
        System::LibraryLoader::GetProcAddress,
    },
    core::s,
};

static mut SHARED: Shared = Shared::new();

#[cfg(not(feature = "dsound"))]
static mut DINPUT: Dinput = Dinput::new();

#[cfg(feature = "dsound")]
static mut DSOUND: Dsound = Dsound::new();

struct Shared {
    DllCanUnloadNow: FARPROC,
    DllGetClassObject: FARPROC,
}

impl Shared {
    const fn new() -> Self {
        Self {
            DllCanUnloadNow: FARPROC::None,
            DllGetClassObject: FARPROC::None,
        }
    }

    fn init(&mut self, dll: HMODULE) {
        unsafe {
            self.DllCanUnloadNow = GetProcAddress(dll, s!("DllCanUnloadNow"));
            self.DllGetClassObject = GetProcAddress(dll, s!("DllGetClassObject"));
        }
    }
}

#[cfg(not(feature = "dsound"))]
struct Dinput {
    DirectInput8Create: FARPROC,
    DllRegisterServer: FARPROC,
    DllUnregisterServer: FARPROC,
    GetdfDIJoystick: FARPROC,
}

#[cfg(not(feature = "dsound"))]
impl Dinput {
    const fn new() -> Self {
        Self {
            DirectInput8Create: FARPROC::None,
            DllRegisterServer: FARPROC::None,
            DllUnregisterServer: FARPROC::None,
            GetdfDIJoystick: FARPROC::None,
        }
    }

    #[allow(static_mut_refs)]
    fn init(&mut self, dll: HMODULE) {
        unsafe {
            SHARED.init(dll);
            self.DirectInput8Create = GetProcAddress(dll, s!("DirectInput8Create"));
            self.DllRegisterServer = GetProcAddress(dll, s!("DllRegisterServer"));
            self.DllUnregisterServer = GetProcAddress(dll, s!("DllUnregisterServer"));
            self.GetdfDIJoystick = GetProcAddress(dll, s!("GetdfDIJoystick"));
        }
    }
}

#[cfg(feature = "dsound")]
struct Dsound {
    DirectSoundCaptureCreate: FARPROC,
    DirectSoundCaptureCreate8: FARPROC,
    DirectSoundCaptureEnumerateA: FARPROC,
    DirectSoundCaptureEnumerateW: FARPROC,
    DirectSoundCreate: FARPROC,
    DirectSoundCreate8: FARPROC,
    DirectSoundEnumerateA: FARPROC,
    DirectSoundEnumerateW: FARPROC,
    DirectSoundFullDuplexCreate: FARPROC,
    GetDeviceID: FARPROC,
}

#[cfg(feature = "dsound")]
impl Dsound {
    const fn new() -> Self {
        Self {
            DirectSoundCaptureCreate: FARPROC::None,
            DirectSoundCaptureCreate8: FARPROC::None,
            DirectSoundCaptureEnumerateA: FARPROC::None,
            DirectSoundCaptureEnumerateW: FARPROC::None,
            DirectSoundCreate: FARPROC::None,
            DirectSoundCreate8: FARPROC::None,
            DirectSoundEnumerateA: FARPROC::None,
            DirectSoundEnumerateW: FARPROC::None,
            DirectSoundFullDuplexCreate: FARPROC::None,
            GetDeviceID: FARPROC::None,
        }
    }

    #[allow(static_mut_refs)]
    fn init(&mut self, dll: HMODULE) {
        unsafe {
            SHARED.init(dll);
            self.DirectSoundCaptureCreate = GetProcAddress(dll, s!("DirectSoundCaptureCreate"));
            self.DirectSoundCaptureCreate8 = GetProcAddress(dll, s!("DirectSoundCaptureCreate8"));
            self.DirectSoundCaptureEnumerateA =
                GetProcAddress(dll, s!("DirectSoundCaptureEnumerateA"));
            self.DirectSoundCaptureEnumerateW =
                GetProcAddress(dll, s!("DirectSoundCaptureEnumerateW"));
            self.DirectSoundCreate = GetProcAddress(dll, s!("DirectSoundCreate"));
            self.DirectSoundCreate8 = GetProcAddress(dll, s!("DirectSoundCreate8"));
            self.DirectSoundEnumerateA = GetProcAddress(dll, s!("DirectSoundEnumerateA"));
            self.DirectSoundEnumerateW = GetProcAddress(dll, s!("DirectSoundEnumerateW"));
            self.DirectSoundFullDuplexCreate =
                GetProcAddress(dll, s!("DirectSoundFullDuplexCreate"));
            self.GetDeviceID = GetProcAddress(dll, s!("GetDeviceID"));
        }
    }
}

// Shared

#[unsafe(no_mangle)]
fn DllCanUnloadNow() {
    unsafe { SHARED.DllCanUnloadNow.unwrap()() };
}
#[unsafe(no_mangle)]
fn DllGetClassObject() {
    unsafe { SHARED.DllGetClassObject.unwrap()() };
}

// dinput8

#[unsafe(no_mangle)]
#[cfg(not(feature = "dsound"))]
fn DirectInput8Create() {
    unsafe { DINPUT.DirectInput8Create.unwrap()() };
}
#[unsafe(no_mangle)]
#[cfg(not(feature = "dsound"))]
fn DllRegisterServer() {
    unsafe { DINPUT.DllRegisterServer.unwrap()() };
}
#[unsafe(no_mangle)]
#[cfg(not(feature = "dsound"))]
fn DllUnregisterServer() {
    unsafe { DINPUT.DllUnregisterServer.unwrap()() };
}
#[unsafe(no_mangle)]
#[cfg(not(feature = "dsound"))]
fn GetdfDIJoystick() {
    unsafe { DINPUT.GetdfDIJoystick.unwrap()() };
}

// dsound

#[unsafe(no_mangle)]
#[cfg(feature = "dsound")]
fn DirectSoundCaptureCreate() {
    unsafe { DSOUND.DirectSoundCaptureCreate.unwrap()() };
}
#[unsafe(no_mangle)]
#[cfg(feature = "dsound")]
fn DirectSoundCaptureCreate8() {
    unsafe { DSOUND.DirectSoundCaptureCreate8.unwrap()() };
}
#[unsafe(no_mangle)]
#[cfg(feature = "dsound")]
fn DirectSoundCaptureEnumerateA() {
    unsafe { DSOUND.DirectSoundCaptureEnumerateA.unwrap()() };
}
#[unsafe(no_mangle)]
#[cfg(feature = "dsound")]
fn DirectSoundCaptureEnumerateW() {
    unsafe { DSOUND.DirectSoundCaptureEnumerateW.unwrap()() };
}
#[unsafe(no_mangle)]
#[cfg(feature = "dsound")]
fn DirectSoundCreate() {
    unsafe { DSOUND.DirectSoundCreate.unwrap()() };
}
#[unsafe(no_mangle)]
#[cfg(feature = "dsound")]
fn DirectSoundCreate8() {
    unsafe { DSOUND.DirectSoundCreate8.unwrap()() };
}
#[unsafe(no_mangle)]
#[cfg(feature = "dsound")]
fn DirectSoundEnumerateA() {
    unsafe { DSOUND.DirectSoundEnumerateA.unwrap()() };
}
#[unsafe(no_mangle)]
#[cfg(feature = "dsound")]
fn DirectSoundEnumerateW() {
    unsafe { DSOUND.DirectSoundEnumerateW.unwrap()() };
}
#[unsafe(no_mangle)]
#[cfg(feature = "dsound")]
fn DirectSoundFullDuplexCreate() {
    unsafe { DSOUND.DirectSoundFullDuplexCreate.unwrap()() };
}
#[unsafe(no_mangle)]
#[cfg(feature = "dsound")]
fn GetDeviceID() {
    unsafe { DSOUND.GetDeviceID.unwrap()() };
}

#[allow(static_mut_refs)]
pub fn init(dll: HMODULE) {
    unsafe {
        cfg_select! {
            feature = "dsound" => DSOUND.init(dll),
            _ => DINPUT.init(dll),
        }
    }
}
