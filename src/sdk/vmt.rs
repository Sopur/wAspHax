use std::mem::{self, transmute};
use libc::c_void;
use std::sync::Mutex;

use crate::util::c_str;

#[allow(non_camel_case_types)]
type intptr = libc::intptr_t;

lazy_static::lazy_static! {
    pub static ref PAGESIZE: i64 = unsafe { libc::sysconf(libc::_SC_PAGE_SIZE) };
    pub static ref PAGEMASK: i64 = !(*PAGESIZE-1);
}

/// Unprotect a memory region - the old protection is returned.
pub unsafe fn unprotect(region: *mut c_void) -> i32 {
    libc::mprotect(
        transmute::<intptr, *mut libc::c_void>(region as intptr & (*PAGEMASK) as intptr),
        *PAGESIZE as usize,
        libc::PROT_READ | libc::PROT_WRITE | libc::PROT_EXEC,
    );

    libc::PROT_READ | libc::PROT_EXEC
}

/// Set the protection of a memory region.
pub unsafe fn protect(region: *mut c_void, protection: i32) {
    libc::mprotect(
        transmute::<intptr, *mut libc::c_void>(region as intptr & (*PAGEMASK) as intptr),
        *PAGESIZE as usize,
        protection,
    );
}

/// Replace an offset in a vtable with a new func.
/// To unhook, just hook again, but replace the new func with the original -
/// as the original is returned when you call hook.
pub unsafe fn hook(instance: *mut c_void, hook: *mut c_void, offset: i32) -> *mut c_void {
    let vtable = *(instance as *mut intptr);
    let entry = vtable + mem::size_of::<intptr>() as intptr * offset as intptr;
    let original = *(entry as *mut isize);

    let original_protection = unprotect(entry as *mut c_void);
    *(entry as *mut isize) = hook as isize;
    protect(entry as *mut c_void, original_protection); // reprotect the unprotected region

    original as *mut c_void
}


pub struct Hook {
    pub original: *mut c_void,
    pub hook: *mut c_void,
    pub offset: i32,
    pub instance: *mut c_void,
}

impl Hook {
    pub fn new(hook: *mut c_void, instance: *mut c_void, offset: i32) -> Self {
        Self {
            original: std::ptr::null_mut(),
            hook: hook as *mut c_void,
            offset,
            instance,
        }
    }

    pub fn hook(&mut self) {
        unsafe {
            self.original = hook(self.instance, self.hook, self.offset);
            log::debug!("hook({:p}, {:p}, 0x{:X})", self.instance, self.hook, self.offset);
            log::debug!("self.original = {:p}", self.original);
        }
    }

    /// Unhooking a hook that has not been hooked is UB.
    pub fn unhook(&mut self) {
        unsafe {
            hook(self.instance, self.original, self.offset);
            log::debug!("unhook({:p}, {:p}, 0x{:X})", self.instance, self.original, self.offset);
        }
    }
}

unsafe impl Send for Hook {}
unsafe impl Sync for Hook {}

lazy_static::lazy_static! {
    pub static ref SDL_SWAP_WIN
    //pub static ref PAINT_TRAVERSE_HOOK: Mutex<Hook> = Mutex::new(Hook::new(paint_traverse as _, super::interfaces::INTERFACES.panel.base as *mut c_void, 41));
}

const fn relative_to_absolute(addr: usize) -> usize {
    unsafe {
        transmute::<isize, usize>(addr as isize + 4 + *(addr as *mut i32) as isize)
    }
}

fn init_sdl() {
    unsafe {
        let lib_sdl = libc::dlopen(c_str!("libSDL2.so.0").as_ptr(), libc::RTLD_LAZY | libc::RTLD_NOLOAD);

        let swap_window_addr: usize = relative_to_absolute(libc::dlsym(lib_sdl, c_str!("SDL_GL_SwapWindow").as_ptr()) as usize + 2);
        let swap_window_addr = swap_window_addr as *mut usize;
        if swap_window_addr == std::ptr::null_mut() {
            log::error!("SDL_GL_SwapWindow not found");
            return;
        } else {
             
        }
    }
}

pub fn init() {
    log::info!("Initializing hooks...");
    
    //PAINT_TRAVERSE_HOOK.lock().unwrap().hook();
}

pub fn cleanup() {
    log::info!("Cleaning up hooks...");
    //PAINT_TRAVERSE_HOOK.lock().unwrap().unhook();
}

unsafe extern "C" fn swap_window_hook(window: *mut sdl2_sys::SDL_Window) {

}

/*type PaintTraverseFn = unsafe extern "C" fn(thisptr: *mut usize, other: *mut usize, panel: u32, force_repaint: bool, allow_force: bool);

unsafe extern "C" fn paint_traverse(thisptr: *mut usize, other: *mut usize, panel: u32, force_repaint: bool, allow_force: bool) {
    use std::ffi::CStr;

    // W2ill be used for drawing
    static mut PANEL_ID: u32 = 0;
    // Will be implemented later for no scope
    static mut PANEL_HUD_ID: u32 = 0;

    let interfaces = &super::interfaces::INTERFACES;

    if PANEL_HUD_ID == 0 {
        let panel_name = interfaces.panel.get_panel_name(panel);

        let c_str = CStr::from_ptr(panel_name);
        let string = c_str.to_str().unwrap();

        if string.contains("HudZoom") {
            PANEL_HUD_ID = panel;
        }
    }

    if PANEL_ID == 0 {
        let panel_name = interfaces.panel.get_panel_name(panel);

        let c_str = CStr::from_ptr(panel_name);
        let string = c_str.to_str().unwrap();

        if string.contains("MatSystemTopPanel") {
            PANEL_ID = panel;
        }
    }

    (transmute::<*mut c_void, PaintTraverseFn>(PAINT_TRAVERSE_HOOK.lock().unwrap().original))(thisptr, other, panel, force_repaint, allow_force);

    if PANEL_ID == panel {
        super::interfaces::surface::draw_box(50, 50, 200, 200, super::interfaces::surface::Color::new_rgb(255, 0, 0));
    }
}*/