use anyhow::{Context, Result, anyhow};
use windows::{
    Win32::{
        Foundation::{FARPROC, HMODULE},
        System::LibraryLoader::GetProcAddress,
    },
    core::s,
};

type WinFunc = unsafe extern "system" fn() -> isize;

unsafe extern "system" fn stub() -> isize {
    0
}

#[derive(Clone, Copy)]
#[repr(transparent)]
struct WinFuncPtr(WinFunc);

impl WinFuncPtr {
    const STUB: WinFuncPtr = WinFuncPtr(stub);

    const fn new(func: WinFunc) -> Self {
        Self(func)
    }

    #[inline]
    fn from_farproc(farproc: FARPROC) -> Result<Self> {
        farproc.try_into()
    }

    #[inline(always)]
    unsafe fn call(self) -> isize {
        unsafe { (self.0)() }
    }
}

impl TryFrom<FARPROC> for WinFuncPtr {
    type Error = anyhow::Error;

    fn try_from(value: FARPROC) -> Result<Self, Self::Error> {
        let func = value.ok_or(anyhow!("FARPROC returned was a None"))?;
        Ok(Self::new(func))
    }
}

static mut SHARED: Shared = Shared::stub();

#[cfg(not(feature = "dsound"))]
static mut DINPUT: Dinput = Dinput::stub();

#[cfg(feature = "dsound")]
static mut DSOUND: Dsound = Dsound::stub();

struct Shared {
    DllCanUnloadNow: WinFuncPtr,
    DllGetClassObject: WinFuncPtr,
}

impl Shared {
    const fn stub() -> Self {
        Self {
            DllCanUnloadNow: WinFuncPtr::STUB,
            DllGetClassObject: WinFuncPtr::STUB,
        }
    }

    fn new(dll: HMODULE) -> Result<Self> {
        let shared = unsafe {
            Self {
                DllCanUnloadNow: WinFuncPtr::from_farproc(GetProcAddress(
                    dll,
                    s!("DllCanUnloadNow"),
                ))?,
                DllGetClassObject: WinFuncPtr::from_farproc(GetProcAddress(
                    dll,
                    s!("DllGetClassObject"),
                ))?,
            }
        };
        Ok(shared)
    }
}

#[cfg(not(feature = "dsound"))]
struct Dinput {
    DirectInput8Create: WinFuncPtr,
    DllRegisterServer: WinFuncPtr,
    DllUnregisterServer: WinFuncPtr,
    GetdfDIJoystick: WinFuncPtr,
}

#[cfg(not(feature = "dsound"))]
impl Dinput {
    const fn stub() -> Self {
        Self {
            DirectInput8Create: WinFuncPtr::STUB,
            DllRegisterServer: WinFuncPtr::STUB,
            DllUnregisterServer: WinFuncPtr::STUB,
            GetdfDIJoystick: WinFuncPtr::STUB,
        }
    }

    fn new(dll: HMODULE) -> Result<Self> {
        let dinput = unsafe {
            Self {
                DirectInput8Create: WinFuncPtr::from_farproc(GetProcAddress(
                    dll,
                    s!("DirectInput8Create"),
                ))?,
                DllRegisterServer: WinFuncPtr::from_farproc(GetProcAddress(
                    dll,
                    s!("DllRegisterServer"),
                ))?,
                DllUnregisterServer: WinFuncPtr::from_farproc(GetProcAddress(
                    dll,
                    s!("DllUnregisterServer"),
                ))?,
                GetdfDIJoystick: WinFuncPtr::from_farproc(GetProcAddress(
                    dll,
                    s!("GetdfDIJoystick"),
                ))?,
            }
        };
        Ok(dinput)
    }
}

#[cfg(feature = "dsound")]
struct Dsound {
    DirectSoundCaptureCreate: WinFuncPtr,
    DirectSoundCaptureCreate8: WinFuncPtr,
    DirectSoundCaptureEnumerateA: WinFuncPtr,
    DirectSoundCaptureEnumerateW: WinFuncPtr,
    DirectSoundCreate: WinFuncPtr,
    DirectSoundCreate8: WinFuncPtr,
    DirectSoundEnumerateA: WinFuncPtr,
    DirectSoundEnumerateW: WinFuncPtr,
    DirectSoundFullDuplexCreate: WinFuncPtr,
    GetDeviceID: WinFuncPtr,
}

#[cfg(feature = "dsound")]
impl Dsound {
    const fn stub() -> Self {
        Self {
            DirectSoundCaptureCreate: WinFuncPtr::STUB,
            DirectSoundCaptureCreate8: WinFuncPtr::STUB,
            DirectSoundCaptureEnumerateA: WinFuncPtr::STUB,
            DirectSoundCaptureEnumerateW: WinFuncPtr::STUB,
            DirectSoundCreate: WinFuncPtr::STUB,
            DirectSoundCreate8: WinFuncPtr::STUB,
            DirectSoundEnumerateA: WinFuncPtr::STUB,
            DirectSoundEnumerateW: WinFuncPtr::STUB,
            DirectSoundFullDuplexCreate: WinFuncPtr::STUB,
            GetDeviceID: WinFuncPtr::STUB,
        }
    }

    fn new(dll: HMODULE) -> Result<Self> {
        let dsound = unsafe {
            Self {
                DirectSoundCaptureCreate: WinFuncPtr::from_farproc(GetProcAddress(
                    dll,
                    s!("DirectSoundCaptureCreate"),
                ))?,
                DirectSoundCaptureCreate8: WinFuncPtr::from_farproc(GetProcAddress(
                    dll,
                    s!("DirectSoundCaptureCreate8"),
                ))?,
                DirectSoundCaptureEnumerateA: WinFuncPtr::from_farproc(GetProcAddress(
                    dll,
                    s!("DirectSoundCaptureEnumerateA"),
                ))?,
                DirectSoundCaptureEnumerateW: WinFuncPtr::from_farproc(GetProcAddress(
                    dll,
                    s!("DirectSoundCaptureEnumerateW"),
                ))?,
                DirectSoundCreate: WinFuncPtr::from_farproc(GetProcAddress(
                    dll,
                    s!("DirectSoundCreate"),
                ))?,
                DirectSoundCreate8: WinFuncPtr::from_farproc(GetProcAddress(
                    dll,
                    s!("DirectSoundCreate8"),
                ))?,
                DirectSoundEnumerateA: WinFuncPtr::from_farproc(GetProcAddress(
                    dll,
                    s!("DirectSoundEnumerateA"),
                ))?,
                DirectSoundEnumerateW: WinFuncPtr::from_farproc(GetProcAddress(
                    dll,
                    s!("DirectSoundEnumerateW"),
                ))?,
                DirectSoundFullDuplexCreate: WinFuncPtr::from_farproc(GetProcAddress(
                    dll,
                    s!("DirectSoundFullDuplexCreate"),
                ))?,
                GetDeviceID: WinFuncPtr::from_farproc(GetProcAddress(dll, s!("GetDeviceID")))?,
            }
        };
        Ok(dsound)
    }
}

// Shared

#[unsafe(no_mangle)]
fn DllCanUnloadNow() -> isize {
    unsafe { SHARED.DllCanUnloadNow.call() }
}
#[unsafe(no_mangle)]
fn DllGetClassObject() -> isize {
    unsafe { SHARED.DllGetClassObject.call() }
}

// dinput8

#[unsafe(no_mangle)]
#[cfg(not(feature = "dsound"))]
fn DirectInput8Create() -> isize {
    unsafe { DINPUT.DirectInput8Create.call() }
}
#[unsafe(no_mangle)]
#[cfg(not(feature = "dsound"))]
fn DllRegisterServer() -> isize {
    unsafe { DINPUT.DllRegisterServer.call() }
}
#[unsafe(no_mangle)]
#[cfg(not(feature = "dsound"))]
fn DllUnregisterServer() -> isize {
    unsafe { DINPUT.DllUnregisterServer.call() }
}
#[unsafe(no_mangle)]
#[cfg(not(feature = "dsound"))]
fn GetdfDIJoystick() -> isize {
    unsafe { DINPUT.GetdfDIJoystick.call() }
}

// dsound

#[unsafe(no_mangle)]
#[cfg(feature = "dsound")]
fn DirectSoundCaptureCreate() -> isize {
    unsafe { DSOUND.DirectSoundCaptureCreate.call() }
}
#[unsafe(no_mangle)]
#[cfg(feature = "dsound")]
fn DirectSoundCaptureCreate8() -> isize {
    unsafe { DSOUND.DirectSoundCaptureCreate8.call() }
}
#[unsafe(no_mangle)]
#[cfg(feature = "dsound")]
fn DirectSoundCaptureEnumerateA() -> isize {
    unsafe { DSOUND.DirectSoundCaptureEnumerateA.call() }
}
#[unsafe(no_mangle)]
#[cfg(feature = "dsound")]
fn DirectSoundCaptureEnumerateW() -> isize {
    unsafe { DSOUND.DirectSoundCaptureEnumerateW.call() }
}
#[unsafe(no_mangle)]
#[cfg(feature = "dsound")]
fn DirectSoundCreate() -> isize {
    unsafe { DSOUND.DirectSoundCreate.call() }
}
#[unsafe(no_mangle)]
#[cfg(feature = "dsound")]
fn DirectSoundCreate8() -> isize {
    unsafe { DSOUND.DirectSoundCreate8.call() }
}
#[unsafe(no_mangle)]
#[cfg(feature = "dsound")]
fn DirectSoundEnumerateA() -> isize {
    unsafe { DSOUND.DirectSoundEnumerateA.call() }
}
#[unsafe(no_mangle)]
#[cfg(feature = "dsound")]
fn DirectSoundEnumerateW() -> isize {
    unsafe { DSOUND.DirectSoundEnumerateW.call() }
}
#[unsafe(no_mangle)]
#[cfg(feature = "dsound")]
fn DirectSoundFullDuplexCreate() -> isize {
    unsafe { DSOUND.DirectSoundFullDuplexCreate.call() }
}
#[unsafe(no_mangle)]
#[cfg(feature = "dsound")]
fn GetDeviceID() -> isize {
    unsafe { DSOUND.GetDeviceID.call() }
}

pub fn init(dll: HMODULE) -> Result<()> {
    unsafe {
        SHARED = Shared::new(dll).context("Failed to initialize Shared")?;
        cfg_select! {
            feature = "dsound" => DSOUND = Dsound::new(dll).context("Failed to initialize Dsound")?,
            _ => DINPUT = Dinput::new(dll).context("Failed to initialize Dinput")?,
        }
    }
    Ok(())
}
