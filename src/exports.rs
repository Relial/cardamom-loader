use windows::{
    Win32::{
        Foundation::{FARPROC, HMODULE},
        System::LibraryLoader::GetProcAddress,
    },
    core::s,
};

static mut DLLSOUND: DllSound = DllSound::new();

struct DllSound {
    DirectSoundCaptureCreate: FARPROC,
    DirectSoundCaptureCreate8: FARPROC,
    DirectSoundCaptureEnumerateA: FARPROC,
    DirectSoundCaptureEnumerateW: FARPROC,
    DirectSoundCreate: FARPROC,
    DirectSoundCreate8: FARPROC,
    DirectSoundEnumerateA: FARPROC,
    DirectSoundEnumerateW: FARPROC,
    DirectSoundFullDuplexCreate: FARPROC,
    DllCanUnloadNow: FARPROC,
    DllGetClassObject: FARPROC,
    GetDeviceID: FARPROC,
}

impl DllSound {
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
            DllCanUnloadNow: FARPROC::None,
            DllGetClassObject: FARPROC::None,
            GetDeviceID: FARPROC::None,
        }
    }

    fn init(&mut self, dll: HMODULE) {
        unsafe {
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
            self.DllCanUnloadNow = GetProcAddress(dll, s!("DllCanUnloadNow"));
            self.DllGetClassObject = GetProcAddress(dll, s!("DllGetClassObject"));
            self.GetDeviceID = GetProcAddress(dll, s!("GetDeviceID"));
        }
    }
}

#[unsafe(no_mangle)]
fn DirectSoundCaptureCreate() {
    unsafe { DLLSOUND.DirectSoundCaptureCreate.unwrap()() };
}
#[unsafe(no_mangle)]
fn DirectSoundCaptureCreate8() {
    unsafe { DLLSOUND.DirectSoundCaptureCreate8.unwrap()() };
}
#[unsafe(no_mangle)]
fn DirectSoundCaptureEnumerateA() {
    unsafe { DLLSOUND.DirectSoundCaptureEnumerateA.unwrap()() };
}
#[unsafe(no_mangle)]
fn DirectSoundCaptureEnumerateW() {
    unsafe { DLLSOUND.DirectSoundCaptureEnumerateW.unwrap()() };
}
#[unsafe(no_mangle)]
fn DirectSoundCreate() {
    unsafe { DLLSOUND.DirectSoundCreate.unwrap()() };
}
#[unsafe(no_mangle)]
fn DirectSoundCreate8() {
    unsafe { DLLSOUND.DirectSoundCreate8.unwrap()() };
}
#[unsafe(no_mangle)]
fn DirectSoundEnumerateA() {
    unsafe { DLLSOUND.DirectSoundEnumerateA.unwrap()() };
}
#[unsafe(no_mangle)]
fn DirectSoundEnumerateW() {
    unsafe { DLLSOUND.DirectSoundEnumerateW.unwrap()() };
}
#[unsafe(no_mangle)]
fn DirectSoundFullDuplexCreate() {
    unsafe { DLLSOUND.DirectSoundFullDuplexCreate.unwrap()() };
}
#[unsafe(no_mangle)]
fn DllCanUnloadNow() {
    unsafe { DLLSOUND.DllCanUnloadNow.unwrap()() };
}
#[unsafe(no_mangle)]
fn DllGetClassObject() {
    unsafe { DLLSOUND.DllGetClassObject.unwrap()() };
}
#[unsafe(no_mangle)]
fn GetDeviceID() {
    unsafe { DLLSOUND.GetDeviceID.unwrap()() };
}

#[allow(static_mut_refs)]
pub fn init(dll: HMODULE) {
    unsafe { DLLSOUND.init(dll) };
}
