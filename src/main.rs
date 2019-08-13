//comment out for println! to work
//#![windows_subsystem = "windows"]

use std::{
    convert::TryInto,
    ffi::OsStr,
    iter::once,
    mem::{size_of, zeroed},
    os::windows::ffi::OsStrExt,
    ptr::null_mut,
};
use winapi::ctypes::c_void;
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::winuser::MessageBoxA;
use winapi::um::winuser::ReleaseDC;
use winapi::um::xinput::XInputGetState;

use winapi::um::memoryapi::VirtualAlloc;
use winapi::um::memoryapi::VirtualFree;

use winapi::um::winuser::GetDC;
use winapi::um::winuser::PeekMessageW;
use winapi::um::winuser::PM_REMOVE;
use winapi::um::winuser::WM_QUIT;
use winapi::{
    shared::{
        minwindef::{HINSTANCE, LPARAM, LRESULT, UINT, WPARAM},
        windef::{HDC, HWND, RECT},
    },
    um::{
        wingdi::{
            StretchDIBits, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, RGBQUAD, SRCCOPY,
        },
        winnt::{MEM_COMMIT, MEM_RELEASE, PAGE_READWRITE},
        winuser::{
            BeginPaint, CreateWindowExW, DefWindowProcW, DispatchMessageW, EndPaint, GetClientRect,
            RegisterClassW, TranslateMessage, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, MSG,
            PAINTSTRUCT, VK_DOWN, VK_ESCAPE, VK_LEFT, VK_RIGHT, VK_SPACE, VK_UP, WM_ACTIVATEAPP,
            WM_CLOSE, WM_DESTROY, WM_KEYDOWN, WM_KEYUP, WM_PAINT, WM_SIZE, WM_SYSKEYDOWN,
            WM_SYSKEYUP, WNDCLASSW, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
        },
    },
};

static mut RUNNING: bool = true;

struct Win32OffScreenBuffer {
    memory: *mut c_void,
    width: i32,
    height: i32,
    pitch: i32,
    bytes_per_pixel: i32,
    info: BITMAPINFO,
}

struct Win32WindowDimension {
    width: i32,
    height: i32,
}

fn win32_get_window_dimension(window: HWND) -> Win32WindowDimension {
    unsafe {
        let mut client_rect = zeroed::<RECT>();
        GetClientRect(window, &mut client_rect);
        let width = client_rect.right - client_rect.left;
        let height = client_rect.bottom - client_rect.top;
        Win32WindowDimension { width, height }
    }
}

static mut GLOBAL_BACKBUFFER: Win32OffScreenBuffer = Win32OffScreenBuffer {
    memory: 0 as *mut c_void,
    width: 0,
    height: 0,
    pitch: 0,
    bytes_per_pixel: 4,
    info: BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: 0,
            biWidth: 0,
            biHeight: 0,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB,
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
    },
};

unsafe fn render_weird_gradient(
    buffer: &Win32OffScreenBuffer,
    blue_offset: i32,
    green_offset: i32,
) {
    let mut row = buffer.memory as *mut u8;
    for y in 0..buffer.height {
        let mut pixel = row as *mut [u8; 4]; //array of 4, u8s
        for x in 0..buffer.width {
            *pixel = [(x + blue_offset) as u8, (y + green_offset) as u8, 0, 0];
            pixel = pixel.offset(1); // adds sizeof(pixel), 4
        }
        row = row.offset(buffer.pitch as isize);
    }
}

fn win32_resize_dibsection(buffer: &mut Win32OffScreenBuffer, width: i32, height: i32) {
    if !buffer.memory.is_null() {
        unsafe {
            VirtualFree(buffer.memory, 0, MEM_RELEASE);
        }
    }

    buffer.width = width;
    buffer.height = height;
    buffer.bytes_per_pixel = 4;
    buffer.info.bmiHeader.biSize = size_of::<BITMAPINFOHEADER>() as u32;
    buffer.info.bmiHeader.biWidth = buffer.width;
    buffer.info.bmiHeader.biHeight = -buffer.height;
    buffer.info.bmiHeader.biPlanes = 1;
    buffer.info.bmiHeader.biBitCount = 32;
    buffer.info.bmiHeader.biCompression = BI_RGB;
    buffer.pitch = buffer.width * buffer.bytes_per_pixel;

    let bitmapmemorysize = (buffer.width * buffer.height) * buffer.bytes_per_pixel;
    buffer.memory = unsafe {
        VirtualAlloc(
            null_mut(),
            bitmapmemorysize as usize,
            MEM_COMMIT,
            PAGE_READWRITE,
        )
    };

    unsafe { render_weird_gradient(&buffer, 1280, 0) }
}

fn win32_display_buffer_in_window(
    device_context: HDC,
    window_width: i32,
    window_height: i32,
    buffer: &Win32OffScreenBuffer,
    _x: i32,
    _y: i32,
    _width: i32,
    _height: i32,
) {
    unsafe {
        StretchDIBits(
            device_context,
            0,
            0,
            window_width,
            window_height,
            0,
            0,
            buffer.width,
            buffer.height,
            buffer.memory,
            &buffer.info,
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
        WM_SIZE => 0,
        WM_CLOSE => {
            RUNNING = false;
            0
        }
        WM_ACTIVATEAPP => 0,
        WM_DESTROY => {
            RUNNING = false;
            0
        }
        WM_SYSKEYDOWN | WM_SYSKEYUP | WM_KEYDOWN | WM_KEYUP => {
            let vk_code = wparam as i32;
            let was_down: bool = (lparam & (1 << 30)) != 0;
            let is_down: bool = (lparam & (1 << 31)) == 0;
            if was_down != is_down {
                match vk_code as u8 as char {
                    'W' => {
                        //87 in deci
                        println!("working W");
                    }
                    'A' => {}
                    'S' => {}
                    'D' => {}
                    'Q' => {}
                    'E' => {}
                    _ => match vk_code {
                        VK_UP => {}
                        VK_LEFT => {}
                        VK_DOWN => {}
                        VK_RIGHT => {}
                        VK_ESCAPE => {}
                        VK_SPACE => {}
                        _ => {}
                    },
                }
            };
            0
        }
        WM_PAINT => {
            let mut paint: PAINTSTRUCT = zeroed::<PAINTSTRUCT>();
            let device_context = BeginPaint(window, &mut paint);
            let x = paint.rcPaint.left;
            let y = paint.rcPaint.top;
            let width = paint.rcPaint.right - paint.rcPaint.left;
            let height = paint.rcPaint.bottom - paint.rcPaint.top;

            let dimension = win32_get_window_dimension(window);

            win32_display_buffer_in_window(
                device_context,
                dimension.width,
                dimension.height,
                &GLOBAL_BACKBUFFER,
                x,
                y,
                width,
                height,
            );

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

    unsafe { win32_resize_dibsection(&mut GLOBAL_BACKBUFFER, 1280, 720) };
    let instance = unsafe { GetModuleHandleW(name.as_ptr() as *const u16) as HINSTANCE };

    let wnd_class = WNDCLASSW {
        style: CS_HREDRAW | CS_VREDRAW,
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
        match RegisterClassW(&wnd_class) {
            0 => {
                MessageBoxA(
                    0 as HWND,
                    b"Call to RegisterClassEx failed!\0".as_ptr() as *const i8,
                    b"Win32 Guided Tour\0".as_ptr() as *const i8,
                    0 as UINT,
                );
            }
            _atom => {
                let window = CreateWindowExW(
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
                );
                if window.is_null() {
                } else {
                    RUNNING = true;
                    let mut offset_x = 0;
                    let mut offset_y = 0;

                    while RUNNING {
                        let mut message = zeroed::<MSG>();
                        while PeekMessageW(&mut message, zeroed(), 0, 0, PM_REMOVE) != 0 {
                            if message.message == WM_QUIT {
                                RUNNING = false;
                            }
                            TranslateMessage(&message);
                            DispatchMessageW(&message);
                        }
                        for controller_index in 0..winapi::um::xinput::XUSER_MAX_COUNT {
                            let mut controller_state: winapi::um::xinput::XINPUT_STATE = zeroed();
                            if XInputGetState(controller_index, &mut controller_state)
                                == winapi::shared::winerror::ERROR_SUCCESS
                            {
                                let pad: winapi::um::xinput::XINPUT_GAMEPAD =
                                    controller_state.Gamepad;

                                let up = pad.wButtons & winapi::um::xinput::XINPUT_GAMEPAD_DPAD_UP;
                                let down =
                                    pad.wButtons & winapi::um::xinput::XINPUT_GAMEPAD_DPAD_DOWN;
                                let left =
                                    pad.wButtons & winapi::um::xinput::XINPUT_GAMEPAD_DPAD_LEFT;
                                let right =
                                    pad.wButtons & winapi::um::xinput::XINPUT_GAMEPAD_DPAD_RIGHT;

                                let back = pad.wButtons & winapi::um::xinput::XINPUT_GAMEPAD_BACK;
                                let left_shoulder =
                                    pad.wButtons & winapi::um::xinput::XINPUT_GAMEPAD_LEFT_SHOULDER;
                                let right_shoulder = pad.wButtons
                                    & winapi::um::xinput::XINPUT_GAMEPAD_RIGHT_SHOULDER;
                                let a_button = pad.wButtons & winapi::um::xinput::XINPUT_GAMEPAD_A;
                                let b_button = pad.wButtons & winapi::um::xinput::XINPUT_GAMEPAD_B;
                                let x_button = pad.wButtons & winapi::um::xinput::XINPUT_GAMEPAD_X;
                                let y_button = pad.wButtons & winapi::um::xinput::XINPUT_GAMEPAD_Y;

                                let stick_x = pad.sThumbLX;
                                let stick_y = pad.sThumbLY;
                            }
                        }

                        render_weird_gradient(&GLOBAL_BACKBUFFER, offset_x, offset_y);

                        let device_context = GetDC(window);
                        let dimension = win32_get_window_dimension(window);
                        win32_display_buffer_in_window(
                            device_context,
                            dimension.width,
                            dimension.height,
                            &GLOBAL_BACKBUFFER,
                            0,
                            0,
                            dimension.width,
                            dimension.height,
                        );
                        ReleaseDC(window, device_context);
                        offset_x += 1;
                        offset_y += 2;
                    }
                }
            }
        }
    }
}

fn main() {
    create_window();
}
