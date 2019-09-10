//#![windows_subsystem = "windows"] //comment out for cmd prompt to pop up and println! to work
use core::arch::x86_64::_rdtsc;
#[link(name = "handmade.dll")]
//use crate::*; //import from main.rs, should move this
use handmade::*;
use std::ffi::CString;
use std::mem::transmute;
use std::{
    convert::TryInto,
    ffi::OsStr,
    iter::once,
    mem::{size_of, zeroed},
    os::windows::ffi::OsStrExt,
    ptr::null_mut,
};
use winapi::ctypes::c_void;
use winapi::shared::guiddef::LPCGUID;
use winapi::shared::minwindef::DWORD;
use winapi::shared::minwindef::HMODULE;
use winapi::shared::mmreg::WAVEFORMATEX;
use winapi::shared::mmreg::WAVE_FORMAT_PCM;
use winapi::shared::ntdef::SHORT;
use winapi::shared::winerror::ERROR_DEVICE_NOT_CONNECTED;
use winapi::shared::winerror::SUCCEEDED;
use winapi::um::dsound::IDirectSound;
use winapi::um::dsound::DSBCAPS_PRIMARYBUFFER;
use winapi::um::dsound::DSBPLAY_LOOPING;
use winapi::um::dsound::DSBUFFERDESC;
use winapi::um::dsound::DSSCL_PRIORITY;
use winapi::um::dsound::DS_OK;
use winapi::um::dsound::LPDIRECTSOUND;
use winapi::um::dsound::LPDIRECTSOUNDBUFFER;
use winapi::um::fileapi::CreateFileA;
use winapi::um::fileapi::GetFileSizeEx;
use winapi::um::fileapi::ReadFile;
use winapi::um::fileapi::WriteFile;
use winapi::um::fileapi::CREATE_ALWAYS;
use winapi::um::fileapi::OPEN_EXISTING;
use winapi::um::handleapi::CloseHandle;
use winapi::um::handleapi::INVALID_HANDLE_VALUE;
use winapi::um::libloaderapi::FreeLibrary;
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::libloaderapi::GetProcAddress;
use winapi::um::libloaderapi::LoadLibraryA;
use winapi::um::memoryapi::VirtualAlloc;
use winapi::um::memoryapi::VirtualFree;
use winapi::um::profileapi::QueryPerformanceFrequency;
use winapi::um::synchapi::Sleep;
use winapi::um::winnt::FILE_SHARE_READ;
use winapi::um::winnt::GENERIC_READ;
use winapi::um::winnt::GENERIC_WRITE;
use winapi::um::winnt::LARGE_INTEGER;
use winapi::um::winnt::MEM_RESERVE;
use winapi::um::winnt::PF_RDTSC_INSTRUCTION_AVAILABLE;
use winapi::um::winuser::MessageBoxA;
use winapi::um::winuser::ReleaseDC;
use winapi::um::xinput::XINPUT_STATE;
use winapi::um::xinput::XINPUT_VIBRATION;
use winapi::um::xinput::XUSER_MAX_COUNT;

use winapi::shared::ntdef::HRESULT;
use winapi::um::mmsystem::TIMERR_NOERROR;
use winapi::um::profileapi::QueryPerformanceCounter;
use winapi::um::timeapi::timeBeginPeriod;
use winapi::um::unknwnbase::LPUNKNOWN;
use winapi::um::winuser::GetDC;
use winapi::um::winuser::PeekMessageW;
use winapi::um::winuser::PM_REMOVE;
use winapi::um::winuser::WM_QUIT;
use winapi::um::xinput::XINPUT_GAMEPAD_LEFT_THUMB_DEADZONE;
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

macro_rules! cstring {
    ($s:expr) => {{
        use std::ffi::CString;
        CString::new($s).unwrap()
    }};
}

static mut RUNNING: bool = true;
static mut GlobalPause: bool = false;
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
static mut GLOBAL_SECONDARY_BUFFER: LPDIRECTSOUNDBUFFER = null_mut();
static mut PERF_COUNT_FREQUENCY: i64 = 0;

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
struct win32_sound_output {
    SamplesPerSecond: u32,
    BytesPerSample: u32,
    LatencySampleCount: u32,
    ToneHz: i32,
    RunningSampleIndex: u32,
    SecondaryBufferSize: u32,
    SafetyBytes: u32,
    ToneVolume: i16,
}
#[derive(Default)]
struct win32_debug_time_marker {
    OutputPlayCursor: DWORD,
    OutputWriteCursor: DWORD,
    OutputLocation: DWORD,
    OutputByteCount: DWORD,
    ExpectedFlipPlayCursor: DWORD,

    FlipPlayCursor: DWORD,
    FlipWriteCursor: DWORD,
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

fn Win32DrawSoundBufferMarker(
    Backbuffer: &mut Win32OffScreenBuffer,
    SoundOutput: &mut win32_sound_output,
    C: f32,
    PadX: i32,
    Top: i32,
    Bottom: i32,
    Value: DWORD,
    Color: u32,
) {
    let XReal32: f32 = (C * Value as f32);
    let X = PadX + XReal32 as i32;
    unsafe { Win32DebugDrawVertical(Backbuffer, X, Top, Bottom, Color) };
}

unsafe fn Win32DebugDrawVertical(
    Backbuffer: &mut Win32OffScreenBuffer,
    X: i32,
    mut Top: i32,
    mut Bottom: i32,
    Color: u32,
) {
    if Top <= 0 {
        Top = 0;
    }

    if Bottom > Backbuffer.height {
        Bottom = Backbuffer.height;
    }

    if (X >= 0) && (X < Backbuffer.width) {
        let mut Pixel = (Backbuffer.memory as *mut u8).offset(
            (X * Backbuffer.bytes_per_pixel + Top * Backbuffer.pitch)
                .try_into()
                .unwrap(),
        );

        for y in Top..Bottom {
            *(Pixel as *mut u32) = Color;
            Pixel = Pixel.offset(Backbuffer.pitch.try_into().unwrap());
        }

        /* for(int Y = Top;
            Y < Bottom;
            ++Y)
        {
            *(uint32 *)Pixel = Color;
            Pixel += Backbuffer->Pitch;
        } */
    }
}

fn Win32DebugSyncDisplay(
    Backbuffer: &mut Win32OffScreenBuffer,
    MarkerCount: i32,
    Markers: &[win32_debug_time_marker],
    CurrentMarkerIndex: i32,
    SoundOutput: &mut win32_sound_output,
    TargetSecondsPerFrame: f32,
) {
    let PadX = 16;
    let PadY = 16;

    let LineHeight = 64;

    let C = (Backbuffer.width - 2 * PadX) as f32 / SoundOutput.SecondaryBufferSize as f32;
    for MarkerIndex in 0..MarkerCount {
        let ThisMarker = &Markers[MarkerIndex as usize]; //might cause seg fault
        let PlayColor: DWORD = 0xFFFFFFFF;
        let WriteColor: DWORD = 0xFFFF0000;
        let ExpectedFlipColor: DWORD = 0xFFFFFF00;
        let PlayWindowColor: DWORD = 0xFFFF00FF;

        let mut Top = PadY;
        let mut Bottom = PadY + LineHeight;
        if MarkerIndex == CurrentMarkerIndex {
            Top += LineHeight + PadY;
            Bottom += LineHeight + PadY;

            let FirstTop = Top;

            Win32DrawSoundBufferMarker(
                Backbuffer,
                SoundOutput,
                C,
                PadX,
                Top,
                Bottom,
                ThisMarker.OutputPlayCursor,
                PlayColor,
            );
            Win32DrawSoundBufferMarker(
                Backbuffer,
                SoundOutput,
                C,
                PadX,
                Top,
                Bottom,
                ThisMarker.OutputWriteCursor,
                WriteColor,
            );

            Top += LineHeight + PadY;
            Bottom += LineHeight + PadY;

            Win32DrawSoundBufferMarker(
                Backbuffer,
                SoundOutput,
                C,
                PadX,
                Top,
                Bottom,
                ThisMarker.OutputLocation,
                PlayColor,
            );
            Win32DrawSoundBufferMarker(
                Backbuffer,
                SoundOutput,
                C,
                PadX,
                Top,
                Bottom,
                ThisMarker.OutputLocation + ThisMarker.OutputByteCount,
                WriteColor,
            );

            Top += LineHeight + PadY;
            Bottom += LineHeight + PadY;

            Win32DrawSoundBufferMarker(
                Backbuffer,
                SoundOutput,
                C,
                PadX,
                FirstTop,
                Bottom,
                ThisMarker.ExpectedFlipPlayCursor,
                ExpectedFlipColor,
            );
        }

        Win32DrawSoundBufferMarker(
            Backbuffer,
            SoundOutput,
            C,
            PadX,
            Top,
            Bottom,
            ThisMarker.FlipPlayCursor,
            PlayColor,
        );
        Win32DrawSoundBufferMarker(
            Backbuffer,
            SoundOutput,
            C,
            PadX,
            Top,
            Bottom,
            ThisMarker.FlipPlayCursor + 480 * SoundOutput.BytesPerSample,
            PlayWindowColor,
        );
        Win32DrawSoundBufferMarker(
            Backbuffer,
            SoundOutput,
            C,
            PadX,
            Top,
            Bottom,
            ThisMarker.FlipWriteCursor,
            WriteColor,
        );
    }
}
type GameUpdateAndRender =
    extern "C" fn(memory: &mut GameMemory, input: &mut GameInput, buffer: &mut GameOffScreenBuffer);

type GameGetSoundSamples =
    unsafe extern "C" fn(Memory: &mut GameMemory, SoundBuffer: &mut game_sound_output_buffer);
struct Win32GameCode {
    game_code_dll: HMODULE,
    update_and_render: GameUpdateAndRender,
    get_sound_samples: GameGetSoundSamples,
    is_valid: bool,
}

unsafe fn win32_load_game_code() -> Win32GameCode {
    let game_code_dll = LoadLibraryA(cstring!("handmade.dll").as_ptr());

    let mut game_code = Win32GameCode {
        game_code_dll,
        update_and_render: game_update_and_render,
        get_sound_samples: GameGetSoundSamples,
        is_valid: false,
    };

    if game_code_dll != null_mut() {
        let update = transmute(GetProcAddress(
            game_code_dll,
            cstring!("game_update_and_render").as_ptr(),
        ));

        let get_sound_samples = transmute(GetProcAddress(
            game_code_dll,
            cstring!("GameGetSoundSamples").as_ptr(),
        ));
        game_code.update_and_render = update;
        game_code.get_sound_samples = get_sound_samples;
        println!("LOADED DLL SUCESSFULLY");
        game_code.is_valid = true;
    } else {
        println!("FAILED TO LOAD GAMECODE");
    }
    game_code
}
unsafe fn win32_unload_game_code(game_code: &mut Win32GameCode) {
    if game_code.game_code_dll != null_mut() {
        FreeLibrary(game_code.game_code_dll);
    }
    game_code.is_valid = false;
    game_code.update_and_render = game_update_and_render;
    game_code.get_sound_samples = GameGetSoundSamples;
}

type XInputGetStateFn = extern "system" fn(DWORD, *mut XINPUT_STATE) -> DWORD;
extern "system" fn xinput_get_state_stub(_: DWORD, _: *mut XINPUT_STATE) -> DWORD {
    return ERROR_DEVICE_NOT_CONNECTED;
}
static mut XInputGetState: XInputGetStateFn = xinput_get_state_stub;

type XInputSetStateFn = extern "system" fn(DWORD, *mut XINPUT_VIBRATION) -> DWORD;
extern "system" fn xinput_set_state_stub(_: DWORD, _: *mut XINPUT_VIBRATION) -> DWORD {
    return ERROR_DEVICE_NOT_CONNECTED;
}
static mut XINPUT_SET_STATE: XInputSetStateFn = xinput_set_state_stub;

unsafe fn win32_load_xinput() {
    let mut library = LoadLibraryA(cstring!("xinput1_4.dll").as_ptr());
    if library == 0 as HINSTANCE {
        library = LoadLibraryA(cstring!("xinput9_1_0.dll").as_ptr());
    }
    if library == 0 as HINSTANCE {
        library = LoadLibraryA(cstring!("xinput1_3.dll").as_ptr());
    }

    if library != 0 as HINSTANCE {
        XInputGetState = transmute(GetProcAddress(library, cstring!("XInputGetState").as_ptr()));
        XINPUT_SET_STATE = transmute(GetProcAddress(library, cstring!("XInputSetState").as_ptr()));
    }
}

type DirectSoundCreateFn = fn(LPCGUID, *mut LPDIRECTSOUND, LPUNKNOWN) -> HRESULT;
unsafe fn win32_init_dsound(window: HWND, samples_per_sec: u32, buffersize: i32) {
    let d_sound_library = LoadLibraryA(cstring!("dsound.dll").as_ptr());

    let mut direct_sound = zeroed::<LPDIRECTSOUND>();

    if d_sound_library != null_mut() {
        let direct_sound_create_ptr =
            GetProcAddress(d_sound_library, cstring!("DirectSoundCreate").as_ptr());
        let DirectSoundCreate: DirectSoundCreateFn = transmute(direct_sound_create_ptr);

        if direct_sound_create_ptr != null_mut()
            && SUCCEEDED(DirectSoundCreate(zeroed(), &mut direct_sound, zeroed()))
        {
            let mut wave_format = zeroed::<WAVEFORMATEX>();
            wave_format.wFormatTag = WAVE_FORMAT_PCM;
            wave_format.nChannels = 2;
            wave_format.nSamplesPerSec = samples_per_sec;
            wave_format.wBitsPerSample = 16;
            wave_format.nBlockAlign = (wave_format.nChannels * wave_format.wBitsPerSample) / 8;
            wave_format.nAvgBytesPerSec =
                wave_format.nSamplesPerSec * wave_format.nBlockAlign as DWORD;
            wave_format.cbSize = 0;
            if SUCCEEDED((*direct_sound).SetCooperativeLevel(window, DSSCL_PRIORITY)) {
                //set coperative level ok

                let mut buffer_description = zeroed::<DSBUFFERDESC>();
                buffer_description.dwSize = size_of::<DSBUFFERDESC>().try_into().unwrap();
                buffer_description.dwFlags = DSBCAPS_PRIMARYBUFFER;
                let mut primary_buffer = zeroed::<LPDIRECTSOUNDBUFFER>();

                if SUCCEEDED((*direct_sound).CreateSoundBuffer(
                    &mut buffer_description,
                    &mut primary_buffer,
                    zeroed(),
                )) {
                    if SUCCEEDED((*primary_buffer).SetFormat(&wave_format)) {
                        println!("primary buffer set ok");
                    } else {
                        //to do diagnostic
                    }
                }
            } else {
                //todo logging
            }

            // TODO(casey): DSBCAPS_GETCURRENTPOSITION2
            let mut buffer_desc = zeroed::<DSBUFFERDESC>();
            buffer_desc.dwSize = size_of::<DSBUFFERDESC>().try_into().unwrap();
            buffer_desc.dwFlags = 0x00010000;
            buffer_desc.dwBufferBytes = buffersize.try_into().unwrap();
            buffer_desc.lpwfxFormat = &mut wave_format;
            if SUCCEEDED((*direct_sound).CreateSoundBuffer(
                &mut buffer_desc,
                &mut GLOBAL_SECONDARY_BUFFER,
                null_mut(),
            )) {
                println!("SECOND BUFFER CREATED");
            } else {
                // TODO: logging
            }
        } else {
            //todo logging
        }
    }
}

unsafe fn Win32FillSoundBuffer(
    SoundOutput: *mut win32_sound_output,
    BytesToLock: DWORD,
    BytesToWrite: DWORD,
    SourceBuffer: &mut game_sound_output_buffer,
) {
    let mut Region1 = null_mut();
    let mut Region1Size: DWORD = 0;
    let mut Region2 = null_mut();
    let mut Region2Size: DWORD = 0;

    if SUCCEEDED((*GLOBAL_SECONDARY_BUFFER).Lock(
        BytesToLock,
        BytesToWrite,
        &mut Region1,
        &mut Region1Size,
        &mut Region2,
        &mut Region2Size,
        0,
    )) {
        let mut Dest = Region1 as *mut i16;
        let mut Source = SourceBuffer.samples as *mut i16;

        for sample_index in 0..(Region1Size / (*SoundOutput).BytesPerSample as u32) {
            *Dest = *Source;
            Dest = Dest.offset(1);
            Source = Source.offset(1);

            *Dest = *Source;
            Dest = Dest.offset(1);
            Source = Source.offset(1);

            (*SoundOutput).RunningSampleIndex += 1;
        }

        Dest = Region2 as *mut i16;
        for sample_index in 0..(Region2Size / (*SoundOutput).BytesPerSample as u32) {
            *Dest = *Source;
            Dest = Dest.offset(1);
            Source = Source.offset(1);

            *Dest = *Source;
            Dest = Dest.offset(1);
            Source = Source.offset(1);
            (*SoundOutput).RunningSampleIndex += 1;
        }

        (*GLOBAL_SECONDARY_BUFFER).Unlock(Region1, Region1Size, Region2, Region2Size);
    }
}

unsafe fn Win32ClearSoundBuffer(SoundOutput: *mut win32_sound_output) {
    let mut Region1 = null_mut();
    let mut Region1Size: DWORD = 0;
    let mut Region2 = null_mut();
    let mut Region2Size: DWORD = 0;

    if SUCCEEDED((*GLOBAL_SECONDARY_BUFFER).Lock(
        0,
        (*SoundOutput).SecondaryBufferSize.try_into().unwrap(),
        &mut Region1,
        &mut Region1Size,
        &mut Region2,
        &mut Region2Size,
        0,
    )) {
        let mut Out = Region1 as *mut u8;

        for byte_index in 0..Region1Size {
            //*out++ = 0;
            *Out = 0;
            Out = Out.offset(1);
        }

        Out = Region2 as *mut u8;

        for byte_index in 0..Region2Size {
            *Out = 0;
            Out = Out.offset(1);
        }

        (*GLOBAL_SECONDARY_BUFFER).Unlock(Region1, Region1Size, Region2, Region2Size);
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
    //unsafe { render_weird_gradient(&buffer, 1280, 0) }
}

fn win32_update_window(
    device_context: HDC,
    window_width: i32,
    window_height: i32,
    buffer: &Win32OffScreenBuffer,
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

unsafe fn win32_process_pending_messages(keyboard_controller: &mut GameControllerInput) {
    let mut message = zeroed::<MSG>();
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
                            win32_process_keyboard_message(
                                &mut keyboard_controller.move_up(),
                                is_down,
                            );
                        }
                        'A' => {
                            win32_process_keyboard_message(
                                &mut keyboard_controller.move_left(),
                                is_down,
                            );
                        }
                        'S' => {
                            win32_process_keyboard_message(
                                &mut keyboard_controller.move_down(),
                                is_down,
                            );
                        }
                        'D' => {
                            win32_process_keyboard_message(
                                &mut keyboard_controller.move_right(),
                                is_down,
                            );
                        }
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
                        #[cfg(feature = "handmade_internal")]
                        'P' => {
                            if is_down {
                                GlobalPause = !GlobalPause;
                            }
                        }

                        _ => {}
                    }

                    match vk_code {
                        VK_DOWN => {
                            win32_process_keyboard_message(
                                &mut keyboard_controller.action_down(),
                                is_down,
                            );
                        }
                        VK_UP => {
                            win32_process_keyboard_message(
                                &mut keyboard_controller.action_up(),
                                is_down,
                            );
                        }
                        VK_LEFT => {
                            win32_process_keyboard_message(
                                &mut keyboard_controller.action_left(),
                                is_down,
                            );
                        }
                        VK_RIGHT => {
                            win32_process_keyboard_message(
                                &mut keyboard_controller.action_right(),
                                is_down,
                            );
                        }
                        VK_ESCAPE => {
                            win32_process_keyboard_message(
                                &mut keyboard_controller.start(),
                                is_down,
                            );
                            RUNNING = false;
                        }
                        VK_SPACE => {
                            win32_process_keyboard_message(
                                &mut keyboard_controller.back(),
                                is_down,
                            );
                        }
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
}

unsafe extern "system" fn win32_main_window_callback(
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
            let dimension = win32_get_window_dimension(window);

            win32_update_window(
                device_context,
                dimension.width,
                dimension.height,
                &GLOBAL_BACKBUFFER,
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

fn win32_process_xinput_stickvalue(value: SHORT, dead_zone_threshold: SHORT) -> f32 {
    let mut result = 0.0;
    if value < -dead_zone_threshold {
        result = (value + dead_zone_threshold) as f32 / (32768.0 - dead_zone_threshold as f32)
    } else if value > dead_zone_threshold {
        result = (value - dead_zone_threshold) as f32 / (32767.0 - dead_zone_threshold as f32)
    };
    result
}

unsafe fn win32_get_wall_clock() -> LARGE_INTEGER {
    let mut result = zeroed::<LARGE_INTEGER>();
    QueryPerformanceCounter(&mut result);
    return result;
}
unsafe fn win32_get_seconds_elasped(start: LARGE_INTEGER, end: LARGE_INTEGER) -> f32 {
    ((end.QuadPart() - start.QuadPart()) as f32 / PERF_COUNT_FREQUENCY as f32) //as f32, moving cast out here drastically changes value. look into why
}

fn main() {
    unsafe {
        winmain();
    }
}
pub unsafe extern "system" fn winmain() {
    let game_code = win32_load_game_code();
    let mut perfcounter_frequency_result = zeroed::<LARGE_INTEGER>();
    QueryPerformanceFrequency(&mut perfcounter_frequency_result);
    PERF_COUNT_FREQUENCY = *perfcounter_frequency_result.QuadPart();

    let desired_scheduler_ms = 1;
    let sleep_is_granular = timeBeginPeriod(desired_scheduler_ms) == TIMERR_NOERROR;
    win32_load_xinput();
    let name = win32_string("HandmadeheroWindowClass");
    let title = win32_string("HandmadeHero");
    let instance = unsafe { GetModuleHandleW(name.as_ptr() as *const u16) as HINSTANCE };

    let wnd_class = WNDCLASSW {
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(win32_main_window_callback),
        hInstance: instance,
        lpszClassName: name.as_ptr(),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hIcon: null_mut(),
        hCursor: null_mut(),
        hbrBackground: null_mut(),
        lpszMenuName: null_mut(),
    };

    win32_resize_dibsection(&mut GLOBAL_BACKBUFFER, 1280, 720);

    const monitor_refresh_hz: u32 = 60;
    const game_update_hz: u32 = monitor_refresh_hz / 2;
    let target_seconds_per_frame: f32 = 1.0 / game_update_hz as f32;
    match RegisterClassW(&wnd_class) {
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
            if window.is_null() != true {
                let device_context = GetDC(window);
                let mut SoundOutput = zeroed::<win32_sound_output>();
                SoundOutput.SamplesPerSecond = 48000;
                SoundOutput.BytesPerSample = size_of::<i16>() as u32 * 2;
                SoundOutput.SecondaryBufferSize =
                    SoundOutput.SamplesPerSecond * SoundOutput.BytesPerSample;

                SoundOutput.LatencySampleCount =
                    3 * (SoundOutput.SamplesPerSecond / game_update_hz);
                SoundOutput.SafetyBytes =
                    (SoundOutput.SamplesPerSecond * SoundOutput.BytesPerSample / game_update_hz)
                        / 3;
                win32_init_dsound(
                    window,
                    SoundOutput.SamplesPerSecond as u32,
                    SoundOutput.SecondaryBufferSize.try_into().unwrap(),
                );
                Win32ClearSoundBuffer(&mut SoundOutput);

                (*GLOBAL_SECONDARY_BUFFER).Play(0, 0, DSBPLAY_LOOPING);

                RUNNING = true;

                let samples = VirtualAlloc(
                    null_mut(),
                    SoundOutput.SecondaryBufferSize.try_into().unwrap(),
                    MEM_COMMIT | MEM_RESERVE,
                    PAGE_READWRITE,
                ) as *mut i16;

                RUNNING = true; // TODO
                let mut game_memory = GameMemory {
                    is_initalized: 0,
                    permanent_storage_size: 64 * 1024 * 1024, //64mb ,
                    transient_storage_size: 4 * 1024 * 1024 * 1024, //4gb
                    transient_storage: null_mut() as *mut std::ffi::c_void,
                    permanent_storage: null_mut() as *mut std::ffi::c_void,
                    debug_platform_free_file_memory,
                    debug_platform_read_entire_file,
                    debug_platform_write_entire_file,
                };

                game_memory.permanent_storage = VirtualAlloc(
                    null_mut(),
                    game_memory.permanent_storage_size as usize,
                    MEM_RESERVE | MEM_COMMIT,
                    PAGE_READWRITE,
                ) as *mut std::ffi::c_void;

                game_memory.transient_storage = VirtualAlloc(
                    null_mut(),
                    game_memory.transient_storage_size as usize,
                    MEM_RESERVE | MEM_COMMIT,
                    PAGE_READWRITE,
                ) as *mut std::ffi::c_void;

                if game_memory.permanent_storage != null_mut() && samples != null_mut() {
                    let mut old_input = GameInput::default();
                    let mut new_input = GameInput::default();

                    let mut last_counter = win32_get_wall_clock();
                    let mut FlipWallClock = win32_get_wall_clock();

                    let mut DebugTimeMarkerIndex = 0;
                    let mut DebugTimeMarkers: [win32_debug_time_marker;
                        (game_update_hz / 2) as usize] = Default::default();

                    let AudioLatencyBytes = 0;
                    let AudioLatencySeconds: f32 = 0.0;
                    let mut SoundIsValid = false;

                    let mut last_cycle_count = _rdtsc();
                    while RUNNING {
                        let old_keyboard_controller: &mut GameControllerInput =
                            &mut old_input.controllers[0 as usize];
                        let mut new_keyboard_controller: &mut GameControllerInput =
                            &mut new_input.controllers[0 as usize];
                        *new_keyboard_controller = GameControllerInput::default();
                        (*new_keyboard_controller).is_connected = true as i32;
                        for button_index in 0..new_keyboard_controller.buttons.len() {
                            new_keyboard_controller.buttons[button_index].ended_down =
                                old_keyboard_controller.buttons[button_index].ended_down;
                        }

                        win32_process_pending_messages(&mut new_keyboard_controller);

                        if !GlobalPause {
                            let mut max_controller_count = XUSER_MAX_COUNT;
                            if max_controller_count > (new_input.controllers.len() - 1) as u32 {
                                max_controller_count = (new_input.controllers.len() - 1) as u32;
                            }
                            for controller_index in 0..max_controller_count {
                                let our_controller_index = controller_index + 1;
                                let old_controller =
                                    &mut old_input.controllers[our_controller_index as usize];

                                let mut new_controller: &mut GameControllerInput =
                                    &mut new_input.controllers[our_controller_index as usize];

                                let mut controller_state: winapi::um::xinput::XINPUT_STATE =
                                    zeroed();
                                if XInputGetState(controller_index, &mut controller_state)
                                    == winapi::shared::winerror::ERROR_SUCCESS
                                {
                                    new_controller.is_connected = true as i32;
                                    let pad = &controller_state.Gamepad;

                                    let up =
                                        pad.wButtons & winapi::um::xinput::XINPUT_GAMEPAD_DPAD_UP;
                                    let down =
                                        pad.wButtons & winapi::um::xinput::XINPUT_GAMEPAD_DPAD_DOWN;
                                    let left =
                                        pad.wButtons & winapi::um::xinput::XINPUT_GAMEPAD_DPAD_LEFT;
                                    let right = pad.wButtons
                                        & winapi::um::xinput::XINPUT_GAMEPAD_DPAD_RIGHT;

                                    new_controller.stick_average_x =
                                        win32_process_xinput_stickvalue(
                                            pad.sThumbLX,
                                            XINPUT_GAMEPAD_LEFT_THUMB_DEADZONE,
                                        );

                                    new_controller.stick_average_y =
                                        win32_process_xinput_stickvalue(
                                            pad.sThumbLY,
                                            XINPUT_GAMEPAD_LEFT_THUMB_DEADZONE,
                                        );

                                    if new_controller.stick_average_x != 0.0
                                        || new_controller.stick_average_y != 0.0
                                    {
                                        new_controller.is_analog = true as i32;
                                    }

                                    if up != 0 {
                                        new_controller.stick_average_y = 1.0 as f32;
                                        new_controller.is_analog = false as i32;
                                    }
                                    if down != 0 {
                                        new_controller.stick_average_y = -1.0 as f32;
                                        new_controller.is_analog = false as i32;
                                    }
                                    if left != 0 {
                                        new_controller.stick_average_x = -1.0 as f32;
                                        new_controller.is_analog = false as i32;
                                    }
                                    if right != 0 {
                                        new_controller.stick_average_x = 1.0 as f32;
                                        new_controller.is_analog = false as i32;
                                    }

                                    let threshold = 0.5;

                                    win32_process_xinput_digital_button(
                                        if new_controller.stick_average_x < -threshold {
                                            1
                                        } else {
                                            0
                                        },
                                        &old_controller.move_left(),
                                        1,
                                        &mut new_controller.move_left(),
                                    );

                                    win32_process_xinput_digital_button(
                                        if new_controller.stick_average_x > threshold {
                                            1
                                        } else {
                                            0
                                        },
                                        &old_controller.move_right(),
                                        1,
                                        &mut new_controller.move_right(),
                                    );

                                    win32_process_xinput_digital_button(
                                        if new_controller.stick_average_y < -threshold {
                                            1
                                        } else {
                                            0
                                        },
                                        &old_controller.move_down(),
                                        1,
                                        &mut new_controller.move_down(),
                                    );

                                    win32_process_xinput_digital_button(
                                        if new_controller.stick_average_y > threshold {
                                            1
                                        } else {
                                            0
                                        },
                                        &old_controller.move_up(),
                                        1,
                                        &mut new_controller.move_up(),
                                    );

                                    win32_process_xinput_digital_button(
                                        pad.wButtons.into(),
                                        &old_controller.action_down(),
                                        winapi::um::xinput::XINPUT_GAMEPAD_A.into(),
                                        &mut new_controller.action_down(),
                                    );
                                    win32_process_xinput_digital_button(
                                        pad.wButtons.into(),
                                        &old_controller.action_right(),
                                        winapi::um::xinput::XINPUT_GAMEPAD_B.into(),
                                        &mut new_controller.action_right(),
                                    );

                                    win32_process_xinput_digital_button(
                                        pad.wButtons.into(),
                                        &old_controller.action_left(),
                                        winapi::um::xinput::XINPUT_GAMEPAD_X.into(),
                                        &mut new_controller.action_left(),
                                    );

                                    win32_process_xinput_digital_button(
                                        pad.wButtons.into(),
                                        &old_controller.action_up(),
                                        winapi::um::xinput::XINPUT_GAMEPAD_Y.into(),
                                        &mut new_controller.action_up(),
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

                                    win32_process_xinput_digital_button(
                                        pad.wButtons.into(),
                                        &old_controller.start(),
                                        winapi::um::xinput::XINPUT_GAMEPAD_START.into(),
                                        &mut new_controller.start(),
                                    );

                                    win32_process_xinput_digital_button(
                                        pad.wButtons.into(),
                                        &old_controller.back(),
                                        winapi::um::xinput::XINPUT_GAMEPAD_BACK.into(),
                                        &mut new_controller.back(),
                                    );
                                } else {
                                    new_controller.is_connected = false as i32;
                                }
                            }
                        }
                        let mut buffer = GameOffScreenBuffer {
                            memory: GLOBAL_BACKBUFFER.memory as *mut std::ffi::c_void,
                            height: GLOBAL_BACKBUFFER.height,
                            width: GLOBAL_BACKBUFFER.width,
                            pitch: GLOBAL_BACKBUFFER.pitch,
                        };
                        (game_code.update_and_render)(
                            &mut game_memory,
                            &mut new_input,
                            &mut buffer,
                        );

                        let AudioWallClock = win32_get_wall_clock();
                        let FromBeginToAudioSeconds =
                            win32_get_seconds_elasped(FlipWallClock, AudioWallClock);

                        let mut PlayCursor = 0;
                        let mut WriteCursor = 0;
                        if (*GLOBAL_SECONDARY_BUFFER)
                            .GetCurrentPosition(&mut PlayCursor, &mut WriteCursor)
                            == DS_OK
                        {
                            if !SoundIsValid {
                                SoundOutput.RunningSampleIndex =
                                    WriteCursor / SoundOutput.BytesPerSample;
                                SoundIsValid = true;
                            }
                            let ByteToLock = ((SoundOutput.RunningSampleIndex
                                * SoundOutput.BytesPerSample)
                                % SoundOutput.SecondaryBufferSize);

                            let ExpectedSoundBytesPerFrame = (SoundOutput.SamplesPerSecond
                                * SoundOutput.BytesPerSample)
                                / game_update_hz;
                            let SecondsLeftUntilFlip =
                                (target_seconds_per_frame - FromBeginToAudioSeconds);
                            let ExpectedBytesUntilFlip = ((SecondsLeftUntilFlip
                                / target_seconds_per_frame)
                                * ExpectedSoundBytesPerFrame as f32)
                                as DWORD;

                            let ExpectedFrameBoundaryByte = PlayCursor + ExpectedSoundBytesPerFrame;

                            let mut SafeWriteCursor = WriteCursor;
                            if SafeWriteCursor < PlayCursor {
                                SafeWriteCursor += SoundOutput.SecondaryBufferSize;
                            }
                            SafeWriteCursor += SoundOutput.SafetyBytes;

                            let AudioCardIsLowLatency =
                                (SafeWriteCursor < ExpectedFrameBoundaryByte);

                            let mut TargetCursor = 0;
                            if (AudioCardIsLowLatency) {
                                TargetCursor =
                                    (ExpectedFrameBoundaryByte + ExpectedSoundBytesPerFrame);
                            } else {
                                TargetCursor = (WriteCursor
                                    + ExpectedSoundBytesPerFrame
                                    + SoundOutput.SafetyBytes);
                            }
                            TargetCursor = (TargetCursor % SoundOutput.SecondaryBufferSize);

                            let mut BytesToWrite = 0;
                            if (ByteToLock > TargetCursor) {
                                BytesToWrite = (SoundOutput.SecondaryBufferSize - ByteToLock);
                                BytesToWrite += TargetCursor;
                            } else {
                                BytesToWrite = TargetCursor - ByteToLock;
                            }
                            let mut SoundBuffer = game_sound_output_buffer {
                                SamplesPerSecond: SoundOutput.SamplesPerSecond,
                                SampleCount: BytesToWrite / SoundOutput.BytesPerSample,
                                samples: samples,
                            };
                            (game_code.get_sound_samples)(&mut game_memory, &mut SoundBuffer);

                            /*

                            #if HANDMADE_INTERNAL
                                                        win32_debug_time_marker *Marker = &DebugTimeMarkers[DebugTimeMarkerIndex];
                                                        Marker->OutputPlayCursor = PlayCursor;
                                                        Marker->OutputWriteCursor = WriteCursor;
                                                        Marker->OutputLocation = ByteToLock;
                                                        Marker->OutputByteCount = BytesToWrite;
                                                        Marker->ExpectedFlipPlayCursor = ExpectedFrameBoundaryByte;

                                                        DWORD UnwrappedWriteCursor = WriteCursor;
                                                        if(UnwrappedWriteCursor < PlayCursor)
                                                        {
                                                            UnwrappedWriteCursor += SoundOutput.SecondaryBufferSize;
                                                        }
                                                        AudioLatencyBytes = UnwrappedWriteCursor - PlayCursor;
                                                        AudioLatencySeconds =
                                                            (((real32)AudioLatencyBytes / (real32)SoundOutput.BytesPerSample) /
                                                             (real32)SoundOutput.SamplesPerSecond);

                                                        char TextBuffer[256];
                                                        _snprintf_s(TextBuffer, sizeof(TextBuffer),
                                                                    "BTL:%u TC:%u BTW:%u - PC:%u WC:%u DELTA:%u (%fs)\n",
                                                                    ByteToLock, TargetCursor, BytesToWrite,
                                                                    PlayCursor, WriteCursor, AudioLatencyBytes, AudioLatencySeconds);
                                                        OutputDebugStringA(TextBuffer);
                            #endif
                            */
                            Win32FillSoundBuffer(
                                &mut SoundOutput,
                                ByteToLock,
                                BytesToWrite,
                                &mut SoundBuffer,
                            );
                        } else {
                            SoundIsValid = false;
                        }

                        let work_counter = win32_get_wall_clock();
                        let work_seconds_elasped =
                            win32_get_seconds_elasped(last_counter, work_counter);

                        let mut seconds_elasped_for_frame = work_seconds_elasped;
                        if seconds_elasped_for_frame < target_seconds_per_frame {
                            if sleep_is_granular {
                                let sleep_ms: DWORD = (1000.0
                                    * (target_seconds_per_frame - seconds_elasped_for_frame))
                                    as DWORD;
                                if sleep_ms > 0 {
                                    Sleep(sleep_ms);
                                }
                            }
                            while seconds_elasped_for_frame < target_seconds_per_frame {
                                seconds_elasped_for_frame =
                                    win32_get_seconds_elasped(last_counter, win32_get_wall_clock());
                            }
                        } else {
                            //TODO: MISSED FRAME RATE
                        }
                        let end_counter = win32_get_wall_clock();
                        let ms_per_frame =
                            1000.0 * win32_get_seconds_elasped(last_counter, end_counter);
                        last_counter = end_counter;

                        let dimension = win32_get_window_dimension(window);

                        // TODO(casey): Note, current is wrong on the zero'th index

                        #[cfg(feature = "handmade_internal")]
                        {
                            Win32DebugSyncDisplay(
                                &mut GLOBAL_BACKBUFFER,
                                DebugTimeMarkers.len() as i32,
                                &DebugTimeMarkers,
                                DebugTimeMarkerIndex - 1,
                                &mut SoundOutput,
                                target_seconds_per_frame,
                            );
                        }
                        win32_update_window(
                            device_context,
                            dimension.width,
                            dimension.height,
                            &GLOBAL_BACKBUFFER,
                        );
                        FlipWallClock = win32_get_wall_clock();

                        // NOTE(casey): This is debug code
                        #[cfg(feature = "handmade_internal")]
                        {
                            let mut PlayCursor: DWORD = 0;
                            let mut WriteCursor: DWORD = 0;
                            if (*GLOBAL_SECONDARY_BUFFER)
                                .GetCurrentPosition(&mut PlayCursor, &mut WriteCursor)
                                == DS_OK
                            {
                                let Marker = &mut DebugTimeMarkers[DebugTimeMarkerIndex as usize];
                                Marker.FlipPlayCursor = PlayCursor;
                                Marker.FlipWriteCursor = WriteCursor;
                            }
                        }

                        let temp = new_input;
                        new_input = old_input;
                        old_input = temp;

                        let end_cyle_counter = _rdtsc();
                        let cycles_elapsed = end_cyle_counter - last_cycle_count;;
                        last_cycle_count = end_cyle_counter;

                        let fps = 0.0;
                        let mcpf: i32 = cycles_elapsed as i32 / (1000 * 1000);
                        println!(
                            "{:#?} ms, the fps is : {:#?}, cycles {:#?}",
                            ms_per_frame, fps, mcpf
                        );

                        #[cfg(feature = "handmade_internal")]
                        {
                            DebugTimeMarkerIndex += 1;
                            if DebugTimeMarkerIndex == DebugTimeMarkers.len() as i32 {
                                DebugTimeMarkerIndex = 0;
                            }
                        }
                    }
                }
            } else {
                println!("could not allocate memory");
            }
        }
    }
}
