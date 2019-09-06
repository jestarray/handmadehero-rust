//comment out for println! to work
//#![windows_subsystem = "windows"]
use crate::game_update_and_render;
use crate::*; //import from main.rs, should move this
use core::arch::x86_64::_rdtsc;
use std::ffi::CString;
use std::{
    convert::TryInto,
    ffi::OsStr,
    iter::once,
    mem::{size_of, zeroed},
    os::windows::ffi::OsStrExt,
    ptr::null_mut,
};
use winapi::ctypes::c_void;
use winapi::shared::minwindef::DWORD;
use winapi::shared::mmreg::WAVEFORMATEX;
use winapi::shared::mmreg::WAVE_FORMAT_PCM;
use winapi::shared::winerror::SUCCEEDED;
use winapi::um::dsound::DirectSoundCreate;
use winapi::um::dsound::DSBCAPS_PRIMARYBUFFER;
use winapi::um::dsound::DSBUFFERDESC;
use winapi::um::dsound::LPDIRECTSOUND;
use winapi::um::dsound::LPDIRECTSOUNDBUFFER;
use winapi::um::fileapi::CreateFileA;
use winapi::um::fileapi::GetFileSizeEx;
use winapi::um::fileapi::WriteFile;
use winapi::um::fileapi::CREATE_ALWAYS;
use winapi::um::fileapi::OPEN_EXISTING;
use winapi::um::handleapi::CloseHandle;
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::winnt::FILE_SHARE_READ;
use winapi::um::winnt::GENERIC_READ;
use winapi::um::winnt::GENERIC_WRITE;
use winapi::um::winnt::LARGE_INTEGER;
use winapi::um::winuser::MessageBoxA;
use winapi::um::winuser::ReleaseDC;
use winapi::um::xinput::XInputGetState;
use winapi::um::xinput::XUSER_MAX_COUNT;

use winapi::um::dsound::IDirectSound;
use winapi::um::dsound::DSSCL_PRIORITY;
use winapi::um::fileapi::ReadFile;
use winapi::um::handleapi::INVALID_HANDLE_VALUE;
use winapi::um::memoryapi::VirtualAlloc;
use winapi::um::memoryapi::VirtualFree;
use winapi::um::profileapi::QueryPerformanceFrequency;
use winapi::um::winnt::MEM_RESERVE;
use winapi::um::winnt::PF_RDTSC_INSTRUCTION_AVAILABLE;

use winapi::um::profileapi::QueryPerformanceCounter;
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
            PAINTSTRUCT, VK_DOWN, VK_ESCAPE, VK_F4, VK_LEFT, VK_RIGHT, VK_SPACE, VK_UP,
            WM_ACTIVATEAPP, WM_CLOSE, WM_DESTROY, WM_KEYDOWN, WM_KEYUP, WM_PAINT, WM_SIZE,
            WM_SYSKEYDOWN, WM_SYSKEYUP, WNDCLASSW, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
        },
    },
};

static mut RUNNING: bool = true;

pub unsafe fn debug_platform_read_entire_file(file_name: &str) -> DebugReadFile {
    let file_name = CString::new(file_name).unwrap();
    let mut result = DebugReadFile {
        content_size: 0,
        contents: null_mut(),
    };
    let file_handle = CreateFileA(
        file_name.as_ptr() as *const i8,
        GENERIC_READ,
        FILE_SHARE_READ,
        0 as *mut winapi::um::minwinbase::SECURITY_ATTRIBUTES,
        OPEN_EXISTING,
        0,
        null_mut(),
    );

    println!("THE FILE H VALUE : {:#?}", file_handle);

    if file_handle != INVALID_HANDLE_VALUE {
        let mut file_size = zeroed::<LARGE_INTEGER>();
        if GetFileSizeEx(file_handle, &mut file_size) != 0 {
            result.contents = VirtualAlloc(
                null_mut(),
                *file_size.QuadPart() as usize,
                MEM_RESERVE | MEM_COMMIT,
                PAGE_READWRITE,
            ) as *mut std::ffi::c_void;
            if result.contents != null_mut() {
                let mut bytes_read = zeroed::<DWORD>();
                if ReadFile(
                    file_handle,
                    result.contents as *mut winapi::ctypes::c_void,
                    *file_size.QuadPart() as u32,
                    &mut bytes_read,
                    null_mut(),
                ) != 0
                {
                    result.content_size = *file_size.QuadPart() as u32;
                //file read successfully
                } else {
                    //TODO logging
                    debug_platform_free_file_memory(result.contents);
                    result.contents = null_mut();
                }
            }
        }

        CloseHandle(file_handle);
    }
    return result;
}
pub unsafe fn debug_platform_free_file_memory(memory: *mut std::ffi::c_void) {
    if memory != null_mut() {
        VirtualFree(memory as *mut winapi::ctypes::c_void, 0, MEM_RELEASE);
    }
}
pub unsafe fn debug_platform_write_entire_file(
    file_name: &str,
    memory_size: u32,
    memory: *mut std::ffi::c_void,
) -> bool {
    let file_name = CString::new(file_name).unwrap();
    let mut result = false;
    let file_handle = CreateFileA(
        file_name.as_ptr() as *const i8,
        GENERIC_WRITE,
        0,
        0 as *mut winapi::um::minwinbase::SECURITY_ATTRIBUTES,
        CREATE_ALWAYS,
        0,
        null_mut(),
    );

    if file_handle != INVALID_HANDLE_VALUE {
        let mut bytes_written = zeroed::<DWORD>();
        if WriteFile(
            file_handle,
            memory as *const winapi::ctypes::c_void,
            memory_size,
            &mut bytes_written,
            null_mut(),
        ) != 0
        {
            result = bytes_written == memory_size;
        //file read successfully
        } else {
            //TODO logging
        }

        CloseHandle(file_handle);
    } else {
        //TODO logging
    }
    return result;
}
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

fn win32_init_dsound(window: HWND, buffersize: i32, samples_per_sec: u32) {
    //todo: sound
    unsafe {
        let mut direct_sound = zeroed::<LPDIRECTSOUND>();

        if SUCCEEDED(DirectSoundCreate(zeroed(), &mut direct_sound, zeroed())) {
            if SUCCEEDED((*direct_sound).SetCooperativeLevel(window, DSSCL_PRIORITY)) {
                let mut buffer_description = zeroed::<DSBUFFERDESC>();
                buffer_description.dwFlags = DSBCAPS_PRIMARYBUFFER;
                buffer_description.dwSize = size_of::<DSBUFFERDESC>().try_into().unwrap();
                let mut primary_buffer = zeroed::<LPDIRECTSOUNDBUFFER>();

                if SUCCEEDED((*direct_sound).CreateSoundBuffer(
                    &buffer_description,
                    &mut primary_buffer,
                    zeroed(),
                )) {
                    let mut wave_format = zeroed::<WAVEFORMATEX>();
                    wave_format.wFormatTag = WAVE_FORMAT_PCM;
                    wave_format.nChannels = 2;
                    wave_format.nSamplesPerSec = samples_per_sec;
                    wave_format.nBlockAlign =
                        (wave_format.nChannels * wave_format.wBitsPerSample) / 8;
                    wave_format.nAvgBytesPerSec =
                        wave_format.nSamplesPerSec * wave_format.nBlockAlign as DWORD;
                    wave_format.wBitsPerSample = 16;
                    wave_format.cbSize = 8;
                    if SUCCEEDED((*primary_buffer).SetFormat(&wave_format)) {
                        // finally set the format
                    } else {

                    }
                }
            }
        } else {
        }
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
    // game_update_and_render();
    //unsafe { render_weird_gradient(&buffer, 1280, 0) }
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

pub fn win32_string(value: &str) -> Vec<u16> {
    //use this when passing strings to windows
    OsStr::new(value).encode_wide().chain(once(0)).collect()
}

fn win32_process_keyboard_message(new_state: &mut GameButtonState, is_down: bool) {
    new_state.ended_down = is_down as i32;
    new_state.half_transition_count += 1;
}
fn win32_process_xinput_digital_button(
    xinput_button_state: DWORD,
    old_state: &GameButtonState,
    button_bit: DWORD,
    mut new_state: &mut GameButtonState,
) {
    new_state.ended_down = ((xinput_button_state & button_bit) == button_bit) as i32;
    new_state.half_transition_count = if old_state.ended_down != new_state.ended_down {
        1
    } else {
        0
    };
}

pub fn create_window() {
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
        let mut prefcounter_frequency_result = zeroed::<LARGE_INTEGER>();
        QueryPerformanceFrequency(&mut prefcounter_frequency_result);
        let prefcounter_frequency = prefcounter_frequency_result.QuadPart();

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
                    RUNNING = true; // TODO
                                    // win32_init_dsound(window);
                    let mut game_memory = GameMemory {
                        is_initalized: 0,
                        permanent_storage_size: 64 * 1024 * 1024, //64mb ,
                        transient_storage_size: 4 * 1024 * 1024 * 1024, //4gb
                        transient_storage: null_mut() as *mut std::ffi::c_void,
                        permanent_storage: null_mut() as *mut std::ffi::c_void,
                    };

                    game_memory.permanent_storage = VirtualAlloc(
                        null_mut(),
                        game_memory.permanent_storage_size as usize,
                        MEM_RESERVE | MEM_COMMIT,
                        PAGE_READWRITE,
                    ) as *mut std::ffi::c_void;

                    //println!("the GAME PERM STORAGE IS {:#?}", game_memory.permanent_storage);

                    game_memory.transient_storage = VirtualAlloc(
                        null_mut(),
                        game_memory.transient_storage_size as usize,
                        MEM_RESERVE | MEM_COMMIT,
                        PAGE_READWRITE,
                    ) as *mut std::ffi::c_void;

                    let mut last_counter = zeroed::<LARGE_INTEGER>();
                    QueryPerformanceCounter(&mut last_counter);

                    let mut old_input = GameInput::default();
                    let mut new_input = GameInput::default();

                    let mut last_cycle_count = _rdtsc();

                    if game_memory.permanent_storage != null_mut() {
                        while RUNNING {
                            let mut message = zeroed::<MSG>();

                            let keyboard_controller: &mut GameControllerInput =
                                &mut new_input.controllers[0 as usize];

                            while PeekMessageW(&mut message, zeroed(), 0, 0, PM_REMOVE) != 0 {
                                if message.message == WM_QUIT {
                                    RUNNING = false;
                                }

                                match message.message {
                                    WM_SYSKEYDOWN | WM_SYSKEYUP | WM_KEYDOWN | WM_KEYUP => {
                                        let vk_code = message.wParam as i32;
                                        let was_down: bool = (message.lParam & (1 << 30)) != 0;
                                        let is_down: bool = (message.lParam & (1 << 31)) == 0;

                                        let alt_is_down: bool = message.lParam & (1 << 29) != 0;
                                        if was_down != is_down {
                                            match vk_code as u8 as char {
                                                'W' => {
                                                    //87 in deci
                                                    println!("working W");
                                                }
                                                'A' => {}
                                                'S' => {}
                                                'D' => {}
                                                'Q' => {
                                                    win32_process_keyboard_message(
                                                        &mut keyboard_controller.left_shoulder(),
                                                        is_down,
                                                    );
                                                }
                                                'E' => {
                                                    win32_process_keyboard_message(
                                                        &mut keyboard_controller.right_shoulder(),
                                                        is_down,
                                                    );
                                                }
                                                _ => {}
                                            }

                                            match vk_code {
                                                VK_DOWN => {
                                                    win32_process_keyboard_message(
                                                        &mut keyboard_controller.down(),
                                                        is_down,
                                                    );
                                                }
                                                VK_UP => {
                                                    win32_process_keyboard_message(
                                                        &mut keyboard_controller.up(),
                                                        is_down,
                                                    );
                                                    println!("UP ARROW WORKING");
                                                }
                                                VK_LEFT => {
                                                    win32_process_keyboard_message(
                                                        &mut keyboard_controller.left(),
                                                        is_down,
                                                    );
                                                }
                                                VK_RIGHT => {
                                                    win32_process_keyboard_message(
                                                        &mut keyboard_controller.right(),
                                                        is_down,
                                                    );
                                                }
                                                VK_ESCAPE => {
                                                    RUNNING = false;
                                                }
                                                VK_SPACE => {}
                                                VK_F4 => {
                                                    if alt_is_down {
                                                        RUNNING = false;
                                                    }
                                                }
                                                _ => {}
                                            }
                                        };
                                    }
                                    _ => {
                                        TranslateMessage(&message);
                                        DispatchMessageW(&message);
                                    }
                                }
                            }

                            let mut max_controller_count = XUSER_MAX_COUNT;
                            if max_controller_count > new_input.controllers.len() as u32 {
                                max_controller_count = new_input.controllers.len() as u32;
                            }
                            for controller_index in 0..max_controller_count {
                                let mut controller_state: winapi::um::xinput::XINPUT_STATE =
                                    zeroed();

                                let old_controller =
                                    &mut old_input.controllers[controller_index as usize];

                                let mut new_controller: &mut GameControllerInput =
                                    &mut new_input.controllers[controller_index as usize];

                                //slow, will get removed in the future
                                if XInputGetState(controller_index, &mut controller_state)
                                    == winapi::shared::winerror::ERROR_SUCCESS
                                {
                                    let pad = &controller_state.Gamepad;

                                    /*      let up =
                                        pad.wButtons & winapi::um::xinput::XINPUT_GAMEPAD_DPAD_UP;
                                    let down =
                                        pad.wButtons & winapi::um::xinput::XINPUT_GAMEPAD_DPAD_DOWN;
                                    let left =
                                        pad.wButtons & winapi::um::xinput::XINPUT_GAMEPAD_DPAD_LEFT;
                                    let right = pad.wButtons
                                        & winapi::um::xinput::XINPUT_GAMEPAD_DPAD_RIGHT; */

                                    let x = if pad.sThumbLX < 0 {
                                        pad.sThumbLX as f32 / 32768.0 as f32
                                    } else {
                                        pad.sThumbLX as f32 / 32767.0 as f32
                                    };

                                    new_controller.end_x = x;
                                    new_controller.max_x = x;
                                    new_controller.min_x = x;

                                    let y = if pad.sThumbLY < 0 {
                                        pad.sThumbLY as f32 / 32768.0 as f32
                                    } else {
                                        pad.sThumbLY as f32 / 32767.0 as f32
                                    };
                                    new_controller.end_y = y;
                                    new_controller.max_y = y;
                                    new_controller.min_y = y;

                                    /* let stick_x = pad.sThumbLX;
                                    let stick_y = pad.sThumbLY; */

                                    win32_process_xinput_digital_button(
                                        pad.wButtons.into(),
                                        &old_controller.down(),
                                        winapi::um::xinput::XINPUT_GAMEPAD_A.into(),
                                        &mut new_controller.down(),
                                    );
                                    win32_process_xinput_digital_button(
                                        pad.wButtons.into(),
                                        &old_controller.right(),
                                        winapi::um::xinput::XINPUT_GAMEPAD_B.into(),
                                        &mut new_controller.right(),
                                    );

                                    win32_process_xinput_digital_button(
                                        pad.wButtons.into(),
                                        &old_controller.left(),
                                        winapi::um::xinput::XINPUT_GAMEPAD_X.into(),
                                        &mut new_controller.left(),
                                    );

                                    win32_process_xinput_digital_button(
                                        pad.wButtons.into(),
                                        &old_controller.up(),
                                        winapi::um::xinput::XINPUT_GAMEPAD_Y.into(),
                                        &mut new_controller.up(),
                                    );

                                    win32_process_xinput_digital_button(
                                        pad.wButtons.into(),
                                        &old_controller.left_shoulder(),
                                        winapi::um::xinput::XINPUT_GAMEPAD_LEFT_SHOULDER.into(),
                                        &mut new_controller.left_shoulder(),
                                    );

                                    win32_process_xinput_digital_button(
                                        pad.wButtons.into(),
                                        &old_controller.right_shoulder(),
                                        winapi::um::xinput::XINPUT_GAMEPAD_RIGHT_SHOULDER.into(),
                                        &mut new_controller.right_shoulder(),
                                    );
                                }
                            }

                            let mut buffer = GameOffScreenBuffer {
                                memory: GLOBAL_BACKBUFFER.memory as *mut std::ffi::c_void,
                                height: GLOBAL_BACKBUFFER.height,
                                width: GLOBAL_BACKBUFFER.width,
                                pitch: GLOBAL_BACKBUFFER.pitch,
                            };
                            game_update_and_render(&mut game_memory, &mut new_input, &mut buffer);

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
                            /*  let end_cyle_counter = _rdtsc();

                            let mut end_counter = zeroed::<LARGE_INTEGER>();
                            QueryPerformanceCounter(&mut end_counter);

                            let cycles_elapsed = end_cyle_counter - last_cycle_count;
                            let counter_elapsed = end_counter.QuadPart() - last_counter.QuadPart();

                            let ms_per_frame = (1000 * counter_elapsed) / prefcounter_frequency;
                            let fps = prefcounter_frequency / counter_elapsed;
                            let mcpf: i32 = cycles_elapsed as i32 / (1000 * 1000);
                            println!(
                                "{:#?} ms, the fps is : {:#?}, cycles {:#?}",
                                ms_per_frame, fps, mcpf
                            );
                            last_counter = end_counter;
                            last_cycle_count = end_cyle_counter; */

                            let temp = new_input;
                            new_input = old_input;
                            old_input = temp;
                        }
                    } else {
                        println!("could not allocate memory");
                    }
                }
            }
        }
    }
}
