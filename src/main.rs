use std::convert::TryInto;
use std::ffi::{c_void, OsStr};

use std::iter::once;
use std::os::windows::ffi::OsStrExt;

use winapi::shared::windef::HDC;

use winapi::um::wingdi::{
    StretchDIBits, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, RGBQUAD, SRCCOPY,
};

use winapi::um::winnt::{MEM_COMMIT, MEM_RELEASE, PAGE_READWRITE};

use kernel32::{GetModuleHandleW, VirtualAlloc, VirtualFree};
use std::mem::{size_of, zeroed};
use std::ptr::null_mut;
use winapi::shared::minwindef::{HINSTANCE, LPARAM, LRESULT, UINT, WPARAM};
use winapi::shared::windef::{HWND, RECT};

use winapi::um::winuser::{
    BeginPaint, CreateWindowExW, DefWindowProcW, DispatchMessageW, EndPaint, GetClientRect,
    GetMessageW, RegisterClassW, TranslateMessage, CS_HREDRAW, CS_OWNDC, CS_VREDRAW, CW_USEDEFAULT,
    MSG, PAINTSTRUCT, WNDCLASSW, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
};

// TODO: Figure out why removing this line makes the window invisible
use winapi::um::winuser::*;

static mut RUNNING: bool = true;

static mut BITMAPMEMORY: *mut c_void = 0 as *mut c_void;
static mut BITMAP_WIDTH: i32 = 0;
static mut BITMAP_HEIGHT: i32 = 0;

static mut BITMAPINFO: BITMAPINFO = BITMAPINFO {
    bmiHeader: BITMAPINFOHEADER {
        biSize: 0,
        biWidth: 0,
        biHeight: 0,
        biPlanes: 0,
        biBitCount: 0,
        biCompression: 0,
        biSizeImage: 0,
        biXPelsPerMeter: 0,
        biYPelsPerMeter: 0,
        biClrUsed: 0,
        biClrImportant: 0,
    },
    bmiColors: [RGBQUAD {
        rgbBlue: 0,
        rgbGreen: 0,
        rgbRed: 0,
        rgbReserved: 0,
    }],
};

/* #[cfg(windows)]
fn print_message(msg: &str) -> Result<i32, Error> {
    let wide: Vec<u16> = OsStr::new(msg).encode_wide().chain(once(0)).collect();
    let ret = unsafe { MessageBoxW(null_mut(), wide.as_ptr(), wide.as_ptr(), MB_OK) };

    if ret == 0 {
        Err(Error::last_os_error())
    } else {
        Ok(ret)
    }
}
#[cfg(not(windows))]
fn print_message(msg: &str) -> Result<(), Error> {
    println!("{}", msg);
    Ok(())
}
 */
fn win32_resize_dibsection(width: i32, height: i32) {
    unsafe {
        if BITMAPMEMORY != zeroed() {
            VirtualFree(BITMAPMEMORY, 0, MEM_RELEASE);
        }
        BITMAP_WIDTH = width;
        BITMAP_HEIGHT = height;
        BITMAPINFO.bmiHeader.biSize = size_of::<BITMAPINFOHEADER>() as u32;
        BITMAPINFO.bmiHeader.biWidth = BITMAP_WIDTH;
        BITMAPINFO.bmiHeader.biHeight = -BITMAP_HEIGHT;
        BITMAPINFO.bmiHeader.biPlanes = 1;
        BITMAPINFO.bmiHeader.biBitCount = 3;
        BITMAPINFO.bmiHeader.biCompression = BI_RGB;
    }
    let bytes_per_pixel = 4;
    unsafe {
        let bitmapmemorysize = (BITMAP_WIDTH * BITMAP_HEIGHT) * bytes_per_pixel;
        BITMAPMEMORY = VirtualAlloc(
            null_mut(),
            bitmapmemorysize as u64,
            MEM_COMMIT,
            PAGE_READWRITE,
        ) as *mut std::ffi::c_void;

        let mut row = BITMAPMEMORY as *mut u8;
        let pitch = width * bytes_per_pixel;
        for _y in 0..BITMAP_HEIGHT {
            let mut pixel = row as *mut u8;
            for _x in 0..BITMAP_WIDTH {
                *pixel = 255;
                pixel = pixel.offset(1);

                *pixel = 255;
                pixel = pixel.offset(1);
                //println!("running {:?}", *pixel);
                *pixel = 255;
                pixel = pixel.offset(1);

                *pixel = 255;
                pixel = pixel.offset(1);
            }
            row = row.offset(pitch.try_into().unwrap());
        }
    }
}

fn win32_update_window(
    device_context: HDC,
    window_rect: &RECT,
    _x: i32,
    _y: i32,
    _width: i32,
    _height: i32,
) {
    unsafe {
        let window_width = window_rect.right - window_rect.left;
        let window_height = window_rect.bottom - window_rect.top;
        StretchDIBits(
            device_context,
            /*  x,
            y,
            width,
            height,
            x,
            y,
            width,
            height, */
            0,
            0,
            BITMAP_WIDTH,
            BITMAP_HEIGHT,
            0,
            0,
            window_width,
            window_height,
            std::ptr::null(),
            &BITMAPINFO,
            DIB_RGB_COLORS,
            SRCCOPY,
        );
    }
}

unsafe extern "system" fn wnd_proc(
    window: HWND,
    message: UINT,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_SIZE => {
            let mut client_rect = zeroed::<RECT>();
            GetClientRect(window, &mut client_rect);
            let width = client_rect.right - client_rect.left;
            let height = client_rect.bottom - client_rect.top;
            win32_resize_dibsection(width, height);
            0
        }
        WM_CLOSE => {
            RUNNING = false;
            0
        }
        WM_ACTIVATEAPP => 0,
        WM_DESTROY => {
            RUNNING = false;
            0
        }
        WM_PAINT => {
            /*   let dim = wind32_get_window_dimension(window);
            let mut paint = zeroed::<winapi::PAINTSTRUCT>();
            let device_context = BeginPaint(window, &mut paint);
            win32_update_window(device_context, dim.width, dim.height, &mut global_buffer);
            EndPaint(window, &mut paint); */
            let mut paint: PAINTSTRUCT = zeroed::<PAINTSTRUCT>();
            let device_context = BeginPaint(window, &mut paint);
            let x = paint.rcPaint.left;
            let y = paint.rcPaint.top;
            let width = paint.rcPaint.right - paint.rcPaint.left;
            let height = paint.rcPaint.bottom - paint.rcPaint.top;

            let mut client_rect = zeroed::<RECT>();
            GetClientRect(window, &mut client_rect);

            win32_update_window(device_context, &client_rect, x, y, width, height);

            EndPaint(window, &paint);
            0
        }
        _ => DefWindowProcW(window, message, wparam, lparam),
    }
}

fn win32_string(value: &str) -> Vec<u16> {
    //use this when passing strings to windows
    OsStr::new(value).encode_wide().chain(once(0)).collect()
}

fn create_window() {
    let name = win32_string("HandmadeheroWindowClass");
    let title = win32_string("HandmadeHero");

    let instance = unsafe { GetModuleHandleW(name.as_ptr() as *const u16) as HINSTANCE };

    let wnd_class = WNDCLASSW {
        style: CS_OWNDC | CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(wnd_proc),
        hInstance: instance,
        lpszClassName: name.as_ptr(),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hIcon: null_mut(),
        hCursor: null_mut(),
        hbrBackground: null_mut(),
        lpszMenuName: null_mut(),
    };

    unsafe {
        RegisterClassW(&wnd_class);
    }

    let _handle = unsafe {
        CreateWindowExW(
            0,
            name.as_ptr(),
            title.as_ptr(),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            null_mut(),
            null_mut(),
            instance,
            null_mut(),
        )
    };

    unsafe {
        let mut message = zeroed::<MSG>();
        while RUNNING {
            let message_result = GetMessageW(&mut message, 0 as HWND, 0 as u32, 0 as u32);
            if message_result > 0 {
                TranslateMessage(&message);
                DispatchMessageW(&message);
            } else {
                break;
            }
        }
    }
}

fn main() {
    //print_message("Hello, world!").unwrap();

    unsafe {
        let mut s = [1, 2, 3];
        let ptr: *mut u32 = s.as_mut_ptr();

        println!("{}", *ptr.offset(1));
        println!("{}", *ptr.offset(2));
    }

    create_window();
}
