use std::os::raw::c_int;

use crate::util;

pub enum EFontFlags
{
    FontflagNone,
    FontflagItalic = 0x001,
    FontflagUnderline = 0x002,
    FontflagStrikeout = 0x004,
    FontflagSymbol = 0x008,
    FontflagAntialias = 0x010,
    FontflagGaussianblur = 0x020,
    FontflagRotary = 0x040,
    FontflagDropshadow = 0x080,
    FontflagAdditive = 0x100,
    FontflagOutline = 0x200,
    FontflagCustom = 0x400,
    FontflagBitmap = 0x800,
}

#[repr(C)]
pub struct Color {
    r: c_int,
    g: c_int,
    b: c_int,
    a: c_int,
}

impl Color {
    pub fn new_rgba(r: i32, g: i32, b: i32, a: i32) -> Self {
        Self {
            r,g,b,a,
        }
    }

    pub fn new_rgb(r: i32, g: i32, b: i32) -> Self {
        Self {
            r,g,b, a: 255,
        }
    }
}

type SetDrawColorFn = unsafe extern "thiscall" fn(thisptr: *mut usize, r: i32, g: i32, b: i32, a: i32);
type DrawFilledRectFn = unsafe extern "thiscall" fn(thisptr: *mut usize, x: i32, y: i32, x1: i32, y1: i32);
type SetTextPosFn = unsafe extern "thiscall" fn(thisptr: *mut usize, x: i32, y: i32);
type DrawPrintTextFn = unsafe extern "thiscall" fn(thisptr: *mut usize, text: *const u16, len: i32);
type DrawOutlinedRectFn = unsafe extern "thiscall" fn(thisptr: *mut usize, x: i32, y: i32, x1: i32, y1: i32);

#[derive(Debug)]
pub struct CSurface {
    pub base: *mut usize,
}

impl CSurface {
    pub unsafe fn from_raw(addr: *mut usize) -> Self {
        Self {
            base: addr,
        }
    }

    pub fn set_draw_color(&self, color: Color) {
        let vfunc = unsafe { std::mem::transmute::<_, SetDrawColorFn>(util::get_virtual_function(self.base, 15)) };
        unsafe { vfunc(self.base, color.r, color.g, color.b, color.a); }
    }

    pub fn draw_filled_rect(&self, x: i32, y: i32, x1: i32, y1: i32) {
        let vfunc = unsafe { std::mem::transmute::<_, DrawFilledRectFn>(util::get_virtual_function(self.base, 16)) };
        unsafe { vfunc(self.base, x, y, x1, y1); }
    }

    pub fn set_text_pos(&self, x: i32, y: i32) {
        let vfunc = unsafe { std::mem::transmute::<_, SetTextPosFn>(util::get_virtual_function(self.base, 26)) };
        unsafe { vfunc(self.base, x, y); }
    }

    pub fn draw_outlined_rect(&self, x: i32, y: i32, x1: i32, y1: i32) {
        let vfunc = unsafe { std::mem::transmute::<_, DrawOutlinedRectFn>(util::get_virtual_function(self.base, 18)) };
        unsafe { vfunc(self.base, x, y, x1, y1); }
    }
}

impl Default for CSurface {
    fn default() -> Self {
        Self {
            base: std::ptr::null_mut(),
        }
    }
}

pub fn draw_box(x: i32, y: i32, w: i32, h: i32, clr: Color) {
    let interfaces = &super::INTERFACES;

    interfaces.surface.set_draw_color(clr);
    interfaces.surface.draw_outlined_rect(x, y, x + w, y + h);

    interfaces.surface.set_draw_color(Color::new_rgb(0, 0, 0));
    interfaces.surface.draw_outlined_rect(x - 1, y - 1, x + w + 1, y + h + 1);
    interfaces.surface.draw_outlined_rect(x + 1, y + 1, x + w - 1, y + h - 1);
}