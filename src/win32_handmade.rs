//#![windows_subsystem = "windows"] //comment out for cmd prompt to pop up and println! to work
use core::arch::x86_64::_rdtsc;
#[link(name = "handmade.dll")]
use handmade::*;
use std::{
    convert::TryInto,
    ffi::{CString, OsStr},
    iter::once,
    mem::{size_of, transmute, zeroed},
    os::windows::ffi::OsStrExt,
    ptr::null_mut,
    str::from_utf8,
};
use winapi::{
    ctypes::c_void,
    shared::{
        guiddef::LPCGUID,
        minwindef::{DWORD, FALSE, FILETIME, HMODULE, LPVOID, MAX_PATH},
        mmreg::{WAVEFORMATEX, WAVE_FORMAT_PCM},
        ntdef::{HRESULT, SHORT},
        windef::POINT,
        winerror::{ERROR_DEVICE_NOT_CONNECTED, SUCCEEDED},
    },
    um::{
        dsound::{
            DSBCAPS_PRIMARYBUFFER, DSBPLAY_LOOPING, DSBUFFERDESC, DSSCL_PRIORITY, DS_OK,
            LPDIRECTSOUND, LPDIRECTSOUNDBUFFER,
        },
        fileapi::{
            CompareFileTime, CreateFileA, FindClose, FindFirstFileA, GetFileAttributesExA,
            GetFileSizeEx, ReadFile, WriteFile, CREATE_ALWAYS, OPEN_EXISTING,
        },
        wingdi::{PatBlt, BLACKNESS},
    },
};

use winapi::um::{
    handleapi::{CloseHandle, INVALID_HANDLE_VALUE},
    libloaderapi::{
        FreeLibrary, GetModuleFileNameA, GetModuleHandleW, GetProcAddress, LoadLibraryA,
    },
    memoryapi::{MapViewOfFile, VirtualAlloc, VirtualFree, FILE_MAP_ALL_ACCESS},
};

use winapi::um::{
    minwinbase::WIN32_FIND_DATAA,
    mmsystem::TIMERR_NOERROR,
    profileapi::{QueryPerformanceCounter, QueryPerformanceFrequency},
    synchapi::Sleep,
    timeapi::timeBeginPeriod,
    unknwnbase::LPUNKNOWN,
    winbase::{CopyFileA, CreateFileMappingA},
    wingdi::GetDeviceCaps,
};

use winapi::um::{
    wingdi::VREFRESH,
    winnt::{
        LARGE_INTEGER_u, RtlCopyMemory, FILE_SHARE_READ, GENERIC_READ, GENERIC_WRITE, HANDLE,
        LARGE_INTEGER, MEM_RESERVE,
    },
    winuser::{
        wsprintfA, GetCursorPos, GetDC, GetKeyState, PeekMessageW, ReleaseDC, ScreenToClient,
        PM_REMOVE, VK_LBUTTON, VK_MBUTTON, VK_RBUTTON, VK_XBUTTON1, VK_XBUTTON2, WM_QUIT,
    },
};

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
        xinput::{
            XINPUT_GAMEPAD_LEFT_THUMB_DEADZONE, XINPUT_STATE, XINPUT_VIBRATION, XUSER_MAX_COUNT,
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

pub unsafe fn debug_platform_read_entire_file(
    thread: &thread_context,
    file_name: &str,
) -> DebugReadFile {
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
                    debug_platform_free_file_memory(thread, result.contents);
                    result.contents = null_mut();
                }
            }
        }

        CloseHandle(file_handle);
    }
    return result;
}
pub unsafe fn debug_platform_free_file_memory(
    thread: &thread_context,
    memory: *mut std::ffi::c_void,
) {
    if memory != null_mut() {
        VirtualFree(memory as *mut winapi::ctypes::c_void, 0, MEM_RELEASE);
    }
}
pub unsafe fn debug_platform_write_entire_file(
    thread: &thread_context,
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
struct win32_replay_buffer {
    FileHandle: HANDLE,
    MemoryMap: HANDLE,
    FileName: [u8; MAX_PATH],
    MemoryBlock: *mut c_void,
}

impl Default for win32_replay_buffer {
    fn default() -> win32_replay_buffer {
        win32_replay_buffer {
            FileHandle: null_mut(),
            MemoryMap: null_mut(),
            FileName: ['\0' as u8; MAX_PATH],
            MemoryBlock: null_mut(),
        }
    }
}

struct win32_state<'a> {
    TotalSize: u64,
    GameMemoryBlock: *mut c_void,
    ReplayBuffers: [win32_replay_buffer; 4],

    RecordingHandle: HANDLE,
    InputRecordingIndex: i32,

    PlaybackHandle: HANDLE,
    InputPlayingIndex: i32,

    exe_file_name: &'a [u8; MAX_PATH],
    one_past_last_slash: &'a [u8],
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

/* fn Win32DrawSoundBufferMarker(
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
} */
type GameUpdateAndRender = extern "C" fn(
    thread: &thread_context,
    memory: &mut GameMemory,
    input: &mut GameInput,
    buffer: &mut GameOffScreenBuffer,
);

type GameGetSoundSamples = unsafe extern "C" fn(
    thread: &thread_context,
    Memory: &mut GameMemory,
    SoundBuffer: &mut game_sound_output_buffer,
);
struct Win32GameCode {
    game_code_dll: HMODULE,
    update_and_render: GameUpdateAndRender,
    get_sound_samples: GameGetSoundSamples,
    is_valid: bool,
    dll_last_write_time: FILETIME,
}

unsafe fn win32_get_last_write_time(file_name: &str) -> FILETIME {
    let mut last_write_time = zeroed::<FILETIME>();
    let name = cstring!(file_name);
    let mut find_data = zeroed::<WIN32_FIND_DATAA>();
    let find_handle = FindFirstFileA(name.as_ptr(), &mut find_data);
    if find_handle != INVALID_HANDLE_VALUE {
        last_write_time = find_data.ftLastWriteTime;
        FindClose(find_handle);
    }
    //FAILING AT COPYING THE FILE BECAUSE OF CALL TO GETFILEATTRIBUTES?
    /*  let mut data = zeroed::<WIN32_FILE_ATTRIBUTE_DATA>();
    if GetFileAttributesExA(
        name.as_ptr(),
        GetFileExInfoStandard,
        &mut data as *mut WIN32_FILE_ATTRIBUTE_DATA as *mut c_void,
    ) != 0
    {
        last_write_time = data.ftLastWriteTime;
    } */
    return last_write_time;
}

unsafe fn win32_load_game_code(source_dll_name: &str, tmp_dll_name: &str) -> Win32GameCode {
    let source_name = cstring!(source_dll_name);
    let temp_name = cstring!(tmp_dll_name);
    let mut result = Win32GameCode {
        game_code_dll: 0 as HMODULE,
        update_and_render: game_update_and_render,
        get_sound_samples: GameGetSoundSamples,
        is_valid: false,
        dll_last_write_time: win32_get_last_write_time(source_dll_name),
    };
    if CopyFileA(source_name.as_ptr(), temp_name.as_ptr(), FALSE) != 0 {
        println!("FILE COPY SUCESS");
    } else {
        /* use winapi::um::errhandlingapi::GetLastError;
        let x = GetLastError();
        dbg!(x); */
        println!("FILE COPY FAIL");
    }

    result.game_code_dll = LoadLibraryA(temp_name.as_ptr());

    if result.game_code_dll != null_mut() {
        let game_update_and_render = cstring!("game_update_and_render");
        let update = transmute(GetProcAddress(
            result.game_code_dll,
            game_update_and_render.as_ptr(),
        ));

        let game_get_sound_samples = cstring!("GameGetSoundSamples");

        let get_sound_samples = transmute(GetProcAddress(
            result.game_code_dll,
            game_get_sound_samples.as_ptr(),
        ));
        result.update_and_render = update;
        result.get_sound_samples = get_sound_samples;
        println!("LOADED DLL SUCESSFULLY");
        result.is_valid = true;
    } else {
        println!("FAILED TO LOAD GAMECODE");
    }
    result
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
    let xinput1_4 = cstring!("xinput1_4.dll");
    let mut library = LoadLibraryA(xinput1_4.as_ptr());
    if library == 0 as HINSTANCE {
        let xinput9_1_0 = cstring!("xinput9_1_0.dll");
        library = LoadLibraryA(xinput9_1_0.as_ptr());
    }
    if library == 0 as HINSTANCE {
        let xinput1_3 = cstring!("xinput1_3.dll");
        library = LoadLibraryA(xinput1_3.as_ptr());
    }

    if library != 0 as HINSTANCE {
        let xinput_get_state_str = cstring!("XInputGetState");
        XInputGetState = transmute(GetProcAddress(library, xinput_get_state_str.as_ptr()));
        let xinput_set_state_str = cstring!("XInputSetState");
        XINPUT_SET_STATE = transmute(GetProcAddress(library, xinput_set_state_str.as_ptr()));
    }
}

type DirectSoundCreateFn = fn(LPCGUID, *mut LPDIRECTSOUND, LPUNKNOWN) -> HRESULT;
unsafe fn win32_init_dsound(window: HWND, samples_per_sec: u32, buffersize: i32) {
    let dsound_str = cstring!("dsound.dll");
    let d_sound_library = LoadLibraryA(dsound_str.as_ptr());

    let mut direct_sound = zeroed::<LPDIRECTSOUND>();

    if d_sound_library != null_mut() {
        let dsoundcrate_str = cstring!("DirectSoundCreate");
        let direct_sound_create_ptr = GetProcAddress(d_sound_library, dsoundcrate_str.as_ptr());
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
}

fn win32_update_window(
    device_context: HDC,
    window_width: i32,
    window_height: i32,
    buffer: &Win32OffScreenBuffer,
) {
    let OffsetX = 10;
    let OffsetY = 10;

    unsafe {
        PatBlt(device_context, 0, 0, window_width, OffsetY, BLACKNESS);
        PatBlt(
            device_context,
            0,
            OffsetY + buffer.height,
            window_width,
            window_height,
            BLACKNESS,
        );
        PatBlt(device_context, 0, 0, OffsetX, window_height, BLACKNESS);
        PatBlt(
            device_context,
            OffsetX + buffer.width,
            0,
            window_width,
            window_height,
            BLACKNESS,
        );
        //blit 1 to 1 for learning how to code renderer, no scaling
        StretchDIBits(
            device_context,
            OffsetX,
            OffsetY,
            buffer.width,
            buffer.height,
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

unsafe fn Win32GetInputFileLocation(
    State: &[u8],
    input_stream: bool,
    slotindex: i32,
    Dest: &mut [u8],
) {
    let mut temp = ['\0' as u8; 64];
    let name = cstring!("loop_edit_%d_%s.hmi");
    wsprintfA(
        temp.as_mut_ptr() as *mut i8,
        name.as_ptr(),
        slotindex,
        if input_stream { b"input" } else { b"state" },
    );
    Win32BuildEXEPathFileName(State, &temp, Dest);
}
fn Win32GetReplayBuffer<'a>(
    ReplayBuffers: &'a mut [win32_replay_buffer; 4],
    Index: i32,
) -> &'a mut win32_replay_buffer {
    let result = &mut ReplayBuffers[Index as usize];
    return result;
}
unsafe fn Win32BeginRecordingInput(State: &mut win32_state, InputRecordingIndex: i32) {
    let ReplayBuffer = Win32GetReplayBuffer(&mut State.ReplayBuffers, InputRecordingIndex);
    if ReplayBuffer.MemoryBlock != null_mut() {
        State.InputRecordingIndex = InputRecordingIndex;
        let mut file_name = ['\0' as u8; MAX_PATH];
        Win32GetInputFileLocation(
            State.one_past_last_slash,
            true,
            InputRecordingIndex,
            &mut file_name,
        );
        State.RecordingHandle = CreateFileA(
            file_name.as_ptr() as *const i8,
            GENERIC_WRITE,
            0,
            null_mut(),
            CREATE_ALWAYS,
            0,
            null_mut(),
        );

        /*

            #if 0
                LARGE_INTEGER FilePosition;
                FilePosition.QuadPart = State->TotalSize;
                SetFilePointerEx(State->RecordingHandle, FilePosition, 0, FILE_BEGIN);
        #endif
            */

        RtlCopyMemory(
            ReplayBuffer.MemoryBlock,
            State.GameMemoryBlock,
            State.TotalSize.try_into().unwrap(),
        );
    }
}

unsafe fn Win32EndRecordingInput(State: &mut win32_state) {
    CloseHandle(State.RecordingHandle);
    State.InputRecordingIndex = 0;
}

unsafe fn Win32BeginInputPlayBack(State: &mut win32_state, InputPlayingIndex: i32) {
    let ReplayBuffer = Win32GetReplayBuffer(&mut State.ReplayBuffers, InputPlayingIndex);
    if ReplayBuffer.MemoryBlock != null_mut() {
        State.InputPlayingIndex = InputPlayingIndex;
        let mut FileName = ['\0' as u8; MAX_PATH];
        Win32GetInputFileLocation(
            State.one_past_last_slash,
            true,
            InputPlayingIndex,
            &mut FileName,
        );
        State.PlaybackHandle = CreateFileA(
            FileName.as_ptr() as *const i8,
            GENERIC_READ,
            0,
            null_mut(),
            OPEN_EXISTING,
            0,
            null_mut(),
        );

        RtlCopyMemory(
            State.GameMemoryBlock,
            ReplayBuffer.MemoryBlock,
            State.TotalSize.try_into().unwrap(),
        );
    }
}

unsafe fn Win32EndInputPlayBack(State: &mut win32_state) {
    CloseHandle(State.PlaybackHandle);
    State.InputPlayingIndex = 0;
}

unsafe fn Win32RecordInput(State: &mut win32_state, NewInput: &mut GameInput) {
    let mut BytesWritten = 0;
    WriteFile(
        State.RecordingHandle,
        NewInput as *mut GameInput as *mut c_void,
        size_of::<GameInput>().try_into().unwrap(),
        &mut BytesWritten,
        null_mut(),
    );
}

unsafe fn Win32PlayBackInput(State: &mut win32_state, NewInput: &mut GameInput) {
    let mut BytesRead = 0;
    if ReadFile(
        State.PlaybackHandle,
        NewInput as *mut GameInput as *mut c_void,
        size_of::<GameInput>().try_into().unwrap(),
        &mut BytesRead,
        null_mut(),
    ) != 0
    {
        if (BytesRead == 0) {
            // NOTE(casey): We've hit the end of the stream, go back to the beginning
            let PlayingIndex = State.InputPlayingIndex;
            Win32EndInputPlayBack(State);
            Win32BeginInputPlayBack(State, PlayingIndex);
            ReadFile(
                State.PlaybackHandle,
                NewInput as *mut GameInput as *mut c_void,
                size_of::<GameInput>().try_into().unwrap(),
                &mut BytesRead,
                null_mut(),
            );
        }
    }
}

unsafe fn win32_process_pending_messages(
    State: &mut win32_state,
    keyboard_controller: &mut GameControllerInput,
) {
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
                        'L' => {
                            if is_down {
                                if State.InputPlayingIndex == 0 {
                                    if State.InputRecordingIndex == 0 {
                                        Win32BeginRecordingInput(State, 1);
                                    } else {
                                        Win32EndRecordingInput(State);
                                        Win32BeginInputPlayBack(State, 1);
                                    }
                                } else {
                                    Win32EndInputPlayBack(State);
                                }
                            }
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
        WM_ACTIVATEAPP => {
            /*  if wparam == TRUE.try_into().unwrap() {
                SetLayeredWindowAttributes(window, RGB(0, 0, 0), 255, LWA_ALPHA);
            } else {
                SetLayeredWindowAttributes(window, RGB(0, 0, 0), 64, LWA_ALPHA);
            } */
            0
        }
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
    if new_state.ended_down != is_down as i32 {
        new_state.ended_down = is_down as i32;
        new_state.half_transition_count += 1;
    }
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

fn Win32BuildEXEPathFileName(State: &[u8], FileName: &[u8], Dest: &mut [u8]) {
    CatStrings(State, FileName, Dest);
}

fn CatStrings(source_a: &[u8], source_b: &[u8], dest: &mut [u8]) {
    //trim 260 null chars?
    dest[..source_a.len()].copy_from_slice(source_a);
    dest[source_a.len()..source_a.len() + source_b.len()].copy_from_slice(source_b);
}
const fn kilobytes(value: u64) -> u64 {
    value * 1024
}
const fn megabytes(value: u64) -> u64 {
    kilobytes(value) * 1024
}
const fn gigabytes(value: u64) -> u64 {
    megabytes(value) * 1024
}
const fn terabytes(value: u64) -> u64 {
    gigabytes(value) * 1024
}
pub unsafe extern "system" fn winmain() {
    //Win32GetEXEFileName HERE. (SKIPPED TO AVOID MUTABLE REF ON WIN32_STATE)
    let mut exe_file_name: [u8; MAX_PATH] = ['\0' as u8; MAX_PATH];
    let size_of_file_name = GetModuleFileNameA(
        0 as HMODULE,
        exe_file_name.as_mut_ptr() as *mut i8,
        size_of::<[char; MAX_PATH]>().try_into().unwrap(),
    );

    let mut limit = 0;
    let mut one_past_last_slash = &exe_file_name[..limit];
    for (index, scan) in exe_file_name.iter().enumerate() {
        match *scan as char {
            '\\' => {
                limit = index + 1;
            }
            '\0' => {
                one_past_last_slash = &exe_file_name[..limit];
            }
            _ => {}
        }
    }
    //win32getexefilename end

    let mut State = win32_state {
        GameMemoryBlock: null_mut(),
        InputPlayingIndex: 0,
        InputRecordingIndex: 0,
        PlaybackHandle: null_mut(),
        RecordingHandle: null_mut(),
        TotalSize: 0,
        exe_file_name: &exe_file_name,
        one_past_last_slash,
        ReplayBuffers: [
            win32_replay_buffer::default(),
            win32_replay_buffer::default(),
            win32_replay_buffer::default(),
            win32_replay_buffer::default(),
        ],
    };

    /*
    let mut one_past_last_slash = exe_file_name.as_mut_ptr();
    for scan in exe_file_name.iter() {
        let m = *scan as u8 as char;
        if m == '\\' {
            one_past_last_slash = (scan as *const u8).offset(1) as *mut u8;
            dbg!(*one_past_last_slash as u8 as char);
        } else if m == '\0' {
            break;
        }
    } */
    let sp = [one_past_last_slash, b"handmade.dll"].concat();
    let source_game_code_dll_file_name_full_path = from_utf8(&sp).unwrap();

    let tp = &[one_past_last_slash, b"handmade_temp.dll"].concat();
    let temp_game_code_dll_full_path = from_utf8(tp).unwrap();

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

    win32_resize_dibsection(&mut GLOBAL_BACKBUFFER, 960, 540);

    match RegisterClassW(&wnd_class) {
        _atom => {
            let window = CreateWindowExW(
                0, //WS_EX_TOPMOST | WS_EX_LAYERED,
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
                let mut SoundOutput = zeroed::<win32_sound_output>();
                SoundOutput.SamplesPerSecond = 48000;
                SoundOutput.BytesPerSample = size_of::<i16>() as u32 * 2;
                SoundOutput.SecondaryBufferSize =
                    SoundOutput.SamplesPerSecond * SoundOutput.BytesPerSample;

                let mut monitor_refresh_hz: u32 = 60;
                let RefreshDC = GetDC(window);
                let Win32RefreshRate = GetDeviceCaps(RefreshDC, VREFRESH);
                ReleaseDC(window, RefreshDC);
                if Win32RefreshRate > 1 {
                    monitor_refresh_hz = Win32RefreshRate.try_into().unwrap();
                }
                let game_update_hz = monitor_refresh_hz as f32 / 2.0 as f32;
                let target_seconds_per_frame: f32 = 1.0 / game_update_hz as f32;

                SoundOutput.SafetyBytes = ((SoundOutput.SamplesPerSecond as f32
                    * SoundOutput.BytesPerSample as f32
                    / game_update_hz)
                    / 3 as f32) as u32;
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

                let mut base_address = 0 as LPVOID;

                #[cfg(feature = "handmade_internal")]
                {
                    base_address = (2 * 1024 * 1024 * 1024 * 1024 as u64) as LPVOID;
                    //2 terabytes
                }

                RUNNING = true; // TODO
                let mut game_memory = GameMemory {
                    is_initalized: 0,
                    permanent_storage_size: megabytes(64), //64mb ,
                    transient_storage_size: gigabytes(1),  //1gb
                    transient_storage: null_mut() as *mut std::ffi::c_void,
                    permanent_storage: null_mut() as *mut std::ffi::c_void,
                    debug_platform_free_file_memory,
                    debug_platform_read_entire_file,
                    debug_platform_write_entire_file,
                };

                State.TotalSize =
                    game_memory.permanent_storage_size + game_memory.transient_storage_size;
                State.GameMemoryBlock = VirtualAlloc(
                    base_address,
                    State.TotalSize as usize,
                    MEM_RESERVE | MEM_COMMIT,
                    PAGE_READWRITE,
                );
                game_memory.permanent_storage = State.GameMemoryBlock as *mut std::ffi::c_void;

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

                //TEST CODE
                /*  {
                    let mut dest = ['\0' as u8;260];
                     Win32GetInputFileLocation(
                         &State.one_past_last_slash,
                         false,
                         1,
                         &mut dest
                    );
                    State.ReplayBuffers[1].FileName = dest;
                }

                for ReplayBuffer in State.ReplayBuffers.iter() {
                     let mut dest = [b'\0';260];
                     Win32GetInputFileLocation(
                         &State.one_past_last_slash,
                         false,
                         1,
                         &mut dest
                    );
                    ReplayBuffer.FileName = dest; // STORE DEST INTO AN ARRAY, THE LOOP THROUGH DEST AND ASSIGN IT TO REPLAYBUFFER IN DIFFERENT MUTABLE LOOP
                } */

                for (ReplayIndex, ReplayBuffer) in State.ReplayBuffers.iter_mut().enumerate()
                /* (int ReplayIndex = 0;
                ReplayIndex < ArrayCount(Win32State.ReplayBuffers);
                ++ReplayIndex) */
                {
                    //let ReplayBuffer = &mut State.ReplayBuffers[ReplayIndex];

                    // TODO(casey): Recording system still seems to take too long
                    // on record start - find out what Windows is doing and if
                    // we can speed up / defer some of that processing.

                    Win32GetInputFileLocation(
                        &State.one_past_last_slash,
                        false,
                        ReplayIndex.try_into().unwrap(),
                        &mut ReplayBuffer.FileName,
                    );

                    ReplayBuffer.FileHandle = CreateFileA(
                        &ReplayBuffer.FileName as *const [u8] as *const i8,
                        GENERIC_WRITE | GENERIC_READ,
                        0,
                        null_mut(),
                        CREATE_ALWAYS,
                        0,
                        null_mut(),
                    );

                    let mut MaxSize = zeroed::<LARGE_INTEGER>();
                    *MaxSize.QuadPart_mut() = State.TotalSize.try_into().unwrap();
                    let msize: LARGE_INTEGER_u = *MaxSize.u_mut();
                    ReplayBuffer.MemoryMap = CreateFileMappingA(
                        ReplayBuffer.FileHandle,
                        null_mut(),
                        PAGE_READWRITE,
                        msize.HighPart.try_into().unwrap(),
                        msize.LowPart,
                        null_mut(),
                    );
                    ReplayBuffer.MemoryBlock = MapViewOfFile(
                        ReplayBuffer.MemoryMap,
                        FILE_MAP_ALL_ACCESS,
                        0,
                        0,
                        State.TotalSize.try_into().unwrap(),
                    );
                    if ReplayBuffer.MemoryBlock != null_mut() {
                    } else {
                        // TODO(casey): Diagnostic
                    }
                }

                if samples != null_mut()
                    && game_memory.permanent_storage != null_mut()
                    && game_memory.transient_storage != null_mut()
                {
                    let mut old_input = GameInput::default();
                    let mut new_input = GameInput::default();
                    let mut last_counter = win32_get_wall_clock();
                    let mut FlipWallClock = win32_get_wall_clock();

                    let mut DebugTimeMarkerIndex = 0;
                    let mut DebugTimeMarkers: [win32_debug_time_marker; 30] = Default::default();

                    let AudioLatencyBytes = 0;
                    let AudioLatencySeconds: f32 = 0.0;
                    let mut SoundIsValid = false;

                    let mut game = win32_load_game_code(
                        source_game_code_dll_file_name_full_path,
                        temp_game_code_dll_full_path,
                    );
                    let mut last_cycle_count = _rdtsc();
                    while RUNNING {
                        new_input.dtForFrame = target_seconds_per_frame;

                        let new_dll_write_time =
                            win32_get_last_write_time(source_game_code_dll_file_name_full_path);
                        if CompareFileTime(&new_dll_write_time, &game.dll_last_write_time) != 0 {
                            println!("RELOADING THE DLL");
                            win32_unload_game_code(&mut game);
                            game = win32_load_game_code(
                                source_game_code_dll_file_name_full_path,
                                temp_game_code_dll_full_path,
                            );
                        }

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

                        win32_process_pending_messages(&mut State, &mut new_keyboard_controller);

                        if !GlobalPause {
                            let mut MouseP = zeroed::<POINT>();
                            GetCursorPos(&mut MouseP);
                            ScreenToClient(window, &mut MouseP);
                            new_input.MouseX = MouseP.x;
                            new_input.MouseY = MouseP.y;
                            new_input.MouseZ = 0; // TODO(casey): Support mousewheel?
                            win32_process_keyboard_message(
                                &mut new_input.MouseButtons[0],
                                (GetKeyState(VK_LBUTTON) & (1 << 15)) != 0,
                            );
                            win32_process_keyboard_message(
                                &mut new_input.MouseButtons[1],
                                (GetKeyState(VK_MBUTTON) & (1 << 15)) != 0,
                            );
                            win32_process_keyboard_message(
                                &mut new_input.MouseButtons[2],
                                (GetKeyState(VK_RBUTTON) & (1 << 15)) != 0,
                            );
                            win32_process_keyboard_message(
                                &mut new_input.MouseButtons[3],
                                (GetKeyState(VK_XBUTTON1) & (1 << 15)) != 0,
                            );
                            win32_process_keyboard_message(
                                &mut new_input.MouseButtons[4],
                                (GetKeyState(VK_XBUTTON2) & (1 << 15)) != 0,
                            );

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
                                    new_controller.is_analog = old_controller.is_analog;

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

                            let Thread = thread_context::default();

                            let mut buffer = GameOffScreenBuffer {
                                memory: GLOBAL_BACKBUFFER.memory as *mut std::ffi::c_void,
                                height: GLOBAL_BACKBUFFER.height,
                                width: GLOBAL_BACKBUFFER.width,
                                pitch: GLOBAL_BACKBUFFER.pitch,
                                bytes_per_pixel: GLOBAL_BACKBUFFER.bytes_per_pixel,
                            };

                            if State.InputRecordingIndex != 0 {
                                Win32RecordInput(&mut State, &mut new_input);
                            }

                            if State.InputPlayingIndex != 0 {
                                Win32PlayBackInput(&mut State, &mut new_input);
                            }

                            (game.update_and_render)(
                                &Thread,
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
                                let ByteToLock = (SoundOutput.RunningSampleIndex
                                    * SoundOutput.BytesPerSample)
                                    % SoundOutput.SecondaryBufferSize;

                                let ExpectedSoundBytesPerFrame = ((SoundOutput.SamplesPerSecond
                                    * SoundOutput.BytesPerSample)
                                    as f32
                                    / game_update_hz)
                                    as i32;

                                //should it be producing a negative float?
                                let SecondsLeftUntilFlip =
                                    target_seconds_per_frame - FromBeginToAudioSeconds;
                                //dbg!(SecondsLeftUntilFlip);
                                //  dbg!(FromBeginToAudioSeconds);
                                let ExpectedBytesUntilFlip = ((SecondsLeftUntilFlip
                                    / target_seconds_per_frame)
                                    * ExpectedSoundBytesPerFrame as f32)
                                    as i32; //should be u32 but secondsleftuntilflip produces a negative valu causing overflow

                                //todo:(jest) should be + ExpectedBytesUntilFlip but causes add overflow.
                                let ExpectedFrameBoundaryByte =
                                    PlayCursor as i32 + ExpectedBytesUntilFlip;

                                let mut SafeWriteCursor = WriteCursor;
                                if SafeWriteCursor < PlayCursor {
                                    SafeWriteCursor += SoundOutput.SecondaryBufferSize;
                                }
                                SafeWriteCursor += SoundOutput.SafetyBytes;

                                let AudioCardIsLowLatency =
                                    (SafeWriteCursor as i32) < ExpectedFrameBoundaryByte;

                                let mut TargetCursor = 0;
                                if AudioCardIsLowLatency {
                                    TargetCursor =
                                        ExpectedFrameBoundaryByte + ExpectedSoundBytesPerFrame;
                                } else {
                                    TargetCursor = (WriteCursor as i32
                                        + ExpectedSoundBytesPerFrame
                                        + SoundOutput.SafetyBytes as i32);
                                }
                                TargetCursor =
                                    (TargetCursor % SoundOutput.SecondaryBufferSize as i32);

                                let mut BytesToWrite = 0;
                                if ByteToLock > TargetCursor as u32 {
                                    BytesToWrite = (SoundOutput.SecondaryBufferSize - ByteToLock);
                                    BytesToWrite += TargetCursor as u32;
                                } else {
                                    BytesToWrite =
                                        (TargetCursor - ByteToLock as i32).try_into().unwrap();
                                }
                                let mut SoundBuffer = game_sound_output_buffer {
                                    SamplesPerSecond: SoundOutput.SamplesPerSecond,
                                    SampleCount: BytesToWrite / SoundOutput.BytesPerSample,
                                    samples: samples,
                                };
                                (game.get_sound_samples)(
                                    &Thread,
                                    &mut game_memory,
                                    &mut SoundBuffer,
                                );

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
                                    seconds_elasped_for_frame = win32_get_seconds_elasped(
                                        last_counter,
                                        win32_get_wall_clock(),
                                    );
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

                            /*   #[cfg(feature = "handmade_internal")]
                            {
                                Win32DebugSyncDisplay(
                                    &mut GLOBAL_BACKBUFFER,
                                    DebugTimeMarkers.len() as i32,
                                    &DebugTimeMarkers,
                                    DebugTimeMarkerIndex - 1,
                                    &mut SoundOutput,
                                    target_seconds_per_frame,
                                );
                            } */
                            let device_context = GetDC(window);
                            win32_update_window(
                                device_context,
                                dimension.width,
                                dimension.height,
                                &GLOBAL_BACKBUFFER,
                            );
                            ReleaseDC(window, device_context);
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
                                    let Marker =
                                        &mut DebugTimeMarkers[DebugTimeMarkerIndex as usize];
                                    Marker.FlipPlayCursor = PlayCursor;
                                    Marker.FlipWriteCursor = WriteCursor;
                                }
                            }

                            let temp = new_input;
                            new_input = old_input;
                            old_input = temp;

                            /*    let end_cyle_counter = _rdtsc();
                                                       let cycles_elapsed = end_cyle_counter - last_cycle_count;;
                                                       last_cycle_count = end_cyle_counter;

                                                       let fps = 0.0;
                                                       let mcpf: i32 = cycles_elapsed as i32 / (1000 * 1000);
                                                       println!(
                                                           "{:#?} ms, the fps is : {:#?}, cycles {:#?}",
                                                           ms_per_frame, fps, mcpf
                                                       );
                            */
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
}
