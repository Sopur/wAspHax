use std::ffi::{CStr, CString};
use std::mem::transmute;

pub mod baseclient;
pub mod entitylist;
pub mod surface;
pub mod panel;

macro_rules! c_str {
    ($lit:expr) => {
        std::ffi::CStr::from_ptr(concat!($lit, "\0").as_ptr() as *const libc::c_char)
    };
}

type InstantiateInterfaceFn = unsafe extern "C" fn() -> *mut usize;

#[repr(C)]
struct InterfaceReg {
    m_CreateFn: InstantiateInterfaceFn,
    m_pName: *const libc::c_char,
    m_pNext: *mut InterfaceReg,
}

pub fn get_interface<T>(file: &str, name: &str, include_version: bool) -> *mut T {
    log::debug!("get_interface({}, {}, {})", file, name, include_version);
    let file = CString::new(file).unwrap();
    unsafe {
        let lib = libc::dlopen(
            file.as_ptr(),
            libc::RTLD_NOLOAD | libc::RTLD_NOW | libc::RTLD_LOCAL,
        );
        if !lib.is_null() {
            let temp = c_str!("s_pInterfaceRegs").as_ptr();
            let interface_reg =
                *transmute::<*mut libc::c_void, *mut *mut InterfaceReg>(libc::dlsym(lib, temp));

            let c_name = CString::new(name).unwrap();

            let mut cur = interface_reg;
            loop {
                if cur.is_null() {
                    break;
                }

                if (!libc::strstr((*cur).m_pName, c_name.as_ptr()).is_null()
                    && libc::strlen((*cur).m_pName) - 3 == libc::strlen(c_name.as_ptr()))
                    || (include_version
                        && (!libc::strstr((*cur).m_pName, c_name.as_ptr()).is_null()
                            && libc::strlen((*cur).m_pName) == libc::strlen(c_name.as_ptr())))
                {
                    let iface: *mut T = transmute::<*mut usize, *mut T>(((*cur).m_CreateFn)());
                    log::debug!(
                        "{} ({:?}) {:p}",
                        name,
                        CStr::from_ptr((*cur).m_pName),
                        iface
                    );
                    return iface;
                }

                cur = (*cur).m_pNext;
            }
        }
        libc::dlclose(lib);
    }

    panic!("Failed to get interface");
}

pub struct Interfaces {
    pub baseclient: baseclient::CBaseClient,
    pub entitylist: entitylist::CEntityList,
    pub surface: surface::CSurface,
    pub panel: panel::CPanel,
}

unsafe impl Send for Interfaces {}
unsafe impl Sync for Interfaces {}

lazy_static::lazy_static! {
    pub static ref INTERFACES: Interfaces = unsafe {
        log::info!("Lazy loading interfaces...");
        Interfaces {
            baseclient: baseclient::CBaseClient::from_raw(get_interface(
                "./csgo/bin/linux64/client_client.so",
                "VClient",
                false,
            )),
            entitylist: entitylist::CEntityList::from_raw(get_interface(
                "./csgo/bin/linux64/client_client.so",
                "VClientEntityList",
                false,
            )),
            surface: surface::CSurface::from_raw(get_interface(
                "./bin/linux64/vguimatsurface_client.so",
                "VGUI_Surface",
                false,
            )),
            panel: panel::CPanel::from_raw(get_interface(
                "./bin/linux64/vgui2_client.so",
                "VGUI_Panel",
                false,
            )),
        }
    };
}
