//!Simple linux framebuffer abstraction.
//!Examples can be found [here](https://github.com/Roysten/rust-framebuffer/tree/master/examples).

extern crate libc;
extern crate memmap;

use libc::ioctl;

use std::fmt;
use std::io::Write;
use std::fs::{OpenOptions, File};
use std::os::unix::io::AsRawFd;
use std::error::Error;
use std::mem;

use memmap::{Mmap, Protection};

const FBIOGET_VSCREENINFO: libc::c_ulong = 0x4600;
const FBIOPUT_VSCREENINFO: libc::c_ulong = 0x4601;
const FBIOGET_FSCREENINFO: libc::c_ulong = 0x4602;

///Bitfield which is a part of VarScreeninfo.
#[repr(C)]
#[derive(Clone, Debug)]
pub struct Bitfield {
    pub offset: u32,
    pub length: u32,
    pub msb_right: u32,
}

///Struct as defined in /usr/include/linux/fb.h
#[repr(C)]
#[derive(Clone, Debug)]
pub struct VarScreeninfo {
    pub xres: u32,    
    pub yres: u32,
    pub xres_virtual: u32,
    pub yres_virtual: u32,
    pub xoffset: u32,
    pub yoffset: u32,
    pub bits_per_pixel: u32,
    pub grayscale: u32,
    pub red: Bitfield,
    pub green: Bitfield,
    pub blue: Bitfield,
    pub transp: Bitfield,
    pub nonstd: u32,
    pub activate: u32,
    pub height: u32,
    pub width: u32,
    pub accel_flags: u32,
    pub pixclock: u32,
    pub left_margin: u32,
    pub right_margin: u32,
    pub upper_margin: u32,
    pub lower_margin: u32,
    pub hsync_len: u32,
    pub vsync_len: u32,
    pub sync: u32,
    pub vmode: u32,
    pub rotate: u32,
    pub colorspace: u32,
    pub reserved: [u32; 4],
}


///Struct as defined in /usr/include/linux/fb.h Note: type is a keyword in Rust and therefore has been
///changed to fb_type.
#[repr(C)]
#[derive(Clone, Debug)]
pub struct FixScreeninfo {
    pub id: [u8; 16],
    pub smem_start: usize,
    pub smem_len: u32,
    pub fb_type: u32,
    pub type_aux: u32,
    pub visual: u32,
    pub xpanstep: u16,
    pub ypanstep: u16,
    pub ywrapstep: u16,
    pub line_length: u32,
    pub mmio_start: usize,
    pub mmio_len: u32,
    pub accel: u32,
    pub capabilities: u16,
    pub reserved: [u16; 2],
}

impl ::std::default::Default for Bitfield {
    fn default() -> Self { unsafe { ::std::mem::zeroed() } }
}

impl ::std::default::Default for VarScreeninfo {
    fn default() -> Self { unsafe { ::std::mem::zeroed() } }
}

impl ::std::default::Default for FixScreeninfo {
    fn default() -> Self { unsafe { ::std::mem::zeroed() } }
}

///Kind of errors that can occur when dealing with the Framebuffer.
#[derive(Debug)]
pub enum FramebufferErrorKind {
    IoctlFailed,
    IoError,
}

#[derive(Debug)]
pub struct FramebufferError {
    pub kind: FramebufferErrorKind,
    pub details: String,
}

impl FramebufferError {
    fn new(kind: FramebufferErrorKind, details: &str) -> FramebufferError {
        FramebufferError { kind: kind, details: String::from(details) }
    }
}

impl std::error::Error for FramebufferError {
    fn description(&self) -> &str {
        &self.details
    }
}

impl fmt::Display for FramebufferError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.description())
    }
}

impl std::convert::From<std::io::Error> for FramebufferError {
    fn from(err: std::io::Error) -> FramebufferError {
        FramebufferError::new(FramebufferErrorKind::IoError, err.description())
    }
}

///Struct that should be used to work with the framebuffer. Direct usage of `frame` should not be
///necessary.
pub struct Framebuffer {
    pub device: File,
    pub frame: Mmap,
    pub var_screen_info: VarScreeninfo,
    pub fix_screen_info: FixScreeninfo,
}

impl Framebuffer {
    pub fn new(path_to_device: &str) -> Result<Framebuffer, FramebufferError> {
        let device = try!(OpenOptions::new().read(true).write(true).open(path_to_device));

        let var_screen_info = try!(Framebuffer::get_var_screen_info(&device));
        let fix_screen_info = try!(Framebuffer::get_fix_screen_info(&device));

        let frame_length = (fix_screen_info.line_length * var_screen_info.yres) as usize;
        let frame = Mmap::open_with_offset(&device, Protection::ReadWrite, 0, frame_length);
        match frame {
            Ok(frame_result) => 
                Ok(Framebuffer {
                    device: device,
                    frame: frame_result,
                    var_screen_info: var_screen_info,
                    fix_screen_info: fix_screen_info,
                }),
                Err(_) => Err(
                    FramebufferError::new(
                    FramebufferErrorKind::IoError,
                    &format!("Could not map memory! Mem start: {} Mem stop: {}", 0, frame_length))
                ),
        }

    }

    ///Writes a frame to the Framebuffer.
    pub fn write_frame<T>(&mut self, frame: &[T]) {
        unsafe { 
            let bytes = std::slice::from_raw_parts(frame.as_ptr() as *const u8, frame.len() * mem::size_of::<T>());
            self.frame.as_mut_slice().write_all(bytes).unwrap();
        }
    }

    pub fn frame<T>(&self) -> &[T] {
        // returns a slice of the framebuffer frame: useful for screenshots
        // however, "The caller must ensure that the file is not concurrently accessed."
        unsafe {
            let frame = self.frame.as_slice();
            std::slice::from_raw_parts(frame.as_ptr() as * const T, frame.len() * mem::size_of::<u8>())
        }
    }

    pub fn frame_mut<T>(&mut self) -> &mut [T] {
        // returns a mutable slice of the framebuffer frame: useful for direct drawing
        // "The caller must ensure that the file is not concurrently accessed."
        unsafe {
            let frame = self.frame.as_mut_slice();
            std::slice::from_raw_parts_mut(frame.as_mut_ptr() as * mut T, frame.len() * mem::size_of::<u8>())
        }
    }

    pub fn resolution(&self) -> (u32, u32) {
        (self.width(), self.height())
    }

    pub fn width(&self) -> u32 {
        self.var_screen_info.xres
    }

    pub fn height(&self) -> u32 {
        self.var_screen_info.yres
    }

    // returns the frame length in bytes
    pub fn frame_length(&self) -> usize {
        self.frame.len()
    }

    pub fn bits_per_pixel(&self) -> usize {
        self.var_screen_info.bits_per_pixel as usize
    }

    ///Creates a FixScreeninfo struct and fills it using ioctl.
    fn get_fix_screen_info(device: &File) -> Result<FixScreeninfo, FramebufferError> {
        let mut info: FixScreeninfo = Default::default();
        let result = unsafe { ioctl(device.as_raw_fd(), FBIOGET_FSCREENINFO, &mut info) };
        match result {
            -1 => Err(FramebufferError::new(FramebufferErrorKind::IoctlFailed, "Ioctl FBIOGET_FSCREENINFO returned -1")),
            _ => Ok(info),
        }
    }

    ///Creates a VarScreeninfo struct and fills it using ioctl.
    fn get_var_screen_info(device: &File) -> Result<VarScreeninfo, FramebufferError> {
        let mut info: VarScreeninfo = Default::default();
        let result = unsafe { ioctl(device.as_raw_fd(), FBIOGET_VSCREENINFO, &mut info) };
        match result {
            -1 => Err(FramebufferError::new(FramebufferErrorKind::IoctlFailed, "Ioctl FBIOGET_VSCREENINFO returned -1")),
            _ => Ok(info),
        }
    }

    pub fn put_var_screen_info(&self) -> Result<i32, FramebufferError> {
        match unsafe { ioctl(self.device.as_raw_fd(), FBIOPUT_VSCREENINFO, &self.var_screen_info) } {
            -1 => Err(FramebufferError::new(FramebufferErrorKind::IoctlFailed, "Ioctl FBIOPUT_VSCREENINFO returned -1 on write")),
            ret => Ok(ret),
        }
    }
}
