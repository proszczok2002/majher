#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::ffi::{OsStr, OsString};
use std::fs;
use std::io::{BufRead, BufReader, Read};
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Output, Stdio};
use std::ptr::{null, null_mut};
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

type Bool = i32;
type Dword = u32;
type Hbrush = isize;
type Hdc = isize;
type Hkey = isize;
type Hcursor = isize;
type Hicon = isize;
type Hinstance = isize;
type Hmenu = isize;
type Hwnd = isize;
type Lparam = isize;
type Lresult = isize;
type Uint = u32;
type Wparam = usize;

const APP_TITLE: &str = "Majher";
const APP_VERSION: &str = "1.9";
const CHAPTER_CONFIG_FILE: &str = "chapter-detector.txt";
const CHAPTER_MODEL_FILE: &str = "ggml-small-q5_1.bin";
const OBFUSCATION_KEY: u8 = 0xA7;
const UPDATE_BASE_ENCODED: &[u8] = &[
    207, 211, 211, 215, 212, 157, 136, 136, 192, 206, 211, 207, 210, 197, 137, 196, 200, 202, 136,
    215, 213, 200, 212, 221, 196, 221, 200, 204, 149, 151, 151, 149, 136, 202, 198, 205, 207, 194,
    213, 136, 213, 194, 203, 194, 198, 212, 194, 212, 136, 203, 198, 211, 194, 212, 211, 136, 195,
    200, 208, 201, 203, 200, 198, 195, 136,
];

const WM_CREATE: Uint = 0x0001;
const WM_DESTROY: Uint = 0x0002;
const WM_SIZE: Uint = 0x0005;
const WM_SETFOCUS: Uint = 0x0007;
const WM_CLOSE: Uint = 0x0010;
const WM_ERASEBKGND: Uint = 0x0014;
const WM_SETTINGCHANGE: Uint = 0x001A;
const WM_COMMAND: Uint = 0x0111;
const WM_KEYDOWN: Uint = 0x0100;
const WM_SETFONT: Uint = 0x0030;
const WM_GETTEXTLENGTH: Uint = 0x000E;
const WM_CTLCOLOREDIT: Uint = 0x0133;
const WM_CTLCOLORLISTBOX: Uint = 0x0134;
const WM_CTLCOLORBTN: Uint = 0x0135;
const WM_CTLCOLORSTATIC: Uint = 0x0138;
const WM_THEMECHANGED: Uint = 0x031A;
const WM_APP: Uint = 0x8000;
const WM_APP_LOG: Uint = WM_APP + 1;
const WM_APP_DONE: Uint = WM_APP + 2;
const WM_APP_FOCUS_FIRST: Uint = WM_APP + 3;
const WM_APP_PROGRESS: Uint = WM_APP + 4;
const WM_APP_UPDATE_RESULT: Uint = WM_APP + 5;
const WM_APP_NOTICE: Uint = WM_APP + 6;
const WM_USER: Uint = 0x0400;

const WS_OVERLAPPEDWINDOW: Dword = 0x00CF0000;
const WS_CHILD: Dword = 0x40000000;
const WS_VISIBLE: Dword = 0x10000000;
const WS_TABSTOP: Dword = 0x00010000;
const WS_BORDER: Dword = 0x00800000;
const WS_VSCROLL: Dword = 0x00200000;
const WS_HSCROLL: Dword = 0x00100000;

const WS_EX_CONTROLPARENT: Dword = 0x00010000;

const ES_AUTOHSCROLL: Dword = 0x0080;
const ES_AUTOVSCROLL: Dword = 0x0040;
const ES_MULTILINE: Dword = 0x0004;
const ES_NUMBER: Dword = 0x2000;
const ES_READONLY: Dword = 0x0800;
const LBS_NOTIFY: Dword = 0x0001;
const CBS_DROPDOWNLIST: Dword = 0x0003;
const CBS_HASSTRINGS: Dword = 0x0200;
const BS_AUTOCHECKBOX: Dword = 0x0003;
const PBS_SMOOTH: Dword = 0x0001;

const LB_ADDSTRING: Uint = 0x0180;
const LB_DELETESTRING: Uint = 0x0182;
const LB_RESETCONTENT: Uint = 0x0184;
const LB_GETCURSEL: Uint = 0x0188;
const LB_ERR: Lresult = -1;

const CB_ADDSTRING: Uint = 0x0143;
const CB_RESETCONTENT: Uint = 0x014B;
const CBN_SELCHANGE: usize = 1;
const CB_GETCURSEL: Uint = 0x0147;
const CB_SETCURSEL: Uint = 0x014E;
const CB_ERR: Lresult = -1;

const BM_GETCHECK: Uint = 0x00F0;
const BM_SETCHECK: Uint = 0x00F1;
const BST_CHECKED: Lresult = 1;

const EM_SETSEL: Uint = 0x00B1;
const EM_REPLACESEL: Uint = 0x00C2;
const PBM_SETPOS: Uint = WM_USER + 2;
const PBM_SETRANGE32: Uint = WM_USER + 6;

const SW_SHOW: i32 = 5;
const MB_YESNO: Uint = 0x00000004;
const MB_YESNOCANCEL: Uint = 0x00000003;
const MB_ICONINFORMATION: Uint = 0x00000040;
const IDYES: i32 = 6;
const IDNO: i32 = 7;
const CW_USEDEFAULT: i32 = -2147483648i32;
const COLOR_WINDOW: isize = 5;
const DEFAULT_GUI_FONT: i32 = 17;
const IDC_ARROW: usize = 32512;
const VK_SHIFT: i32 = 0x10;
const VK_TAB: Wparam = 0x09;

const OFN_ALLOWMULTISELECT: Dword = 0x00000200;
const OFN_EXPLORER: Dword = 0x00080000;
const OFN_FILEMUSTEXIST: Dword = 0x00001000;
const OFN_HIDEREADONLY: Dword = 0x00000004;

const BIF_RETURNONLYFSDIRS: Uint = 0x0001;
const BIF_NEWDIALOGSTYLE: Uint = 0x0040;
const BIF_EDITBOX: Uint = 0x0010;

const CREATE_NO_WINDOW: Dword = 0x08000000;
const PROCESS_SET_INFORMATION: Dword = 0x0200;
const IDLE_PRIORITY_CLASS: Dword = 0x00000040;
const BELOW_NORMAL_PRIORITY_CLASS: Dword = 0x00004000;
const NORMAL_PRIORITY_CLASS: Dword = 0x00000020;
const COINIT_APARTMENTTHREADED: Dword = 0x00000002;
const CLSCTX_INPROC_SERVER: Dword = 0x00000001;
const OBJID_CLIENT: Dword = 0xFFFF_FFFC;
const CHILDID_SELF: Dword = 0;
const EVENT_OBJECT_VALUECHANGE: Dword = 0x800E;
const ICC_PROGRESS_CLASS: Dword = 0x00000020;
const RRF_RT_REG_DWORD: Dword = 0x00000018;
const HKEY_CURRENT_USER: Hkey = 0x80000001u32 as Hkey;
const DWMWA_USE_IMMERSIVE_DARK_MODE_BEFORE_20H1: Dword = 19;
const DWMWA_USE_IMMERSIVE_DARK_MODE: Dword = 20;
const TRANSPARENT: i32 = 1;
const DARK_BACKGROUND: Dword = 0x00202020;
const DARK_CONTROL_BACKGROUND: Dword = 0x002D2D2D;
const DARK_TEXT: Dword = 0x00F0F0F0;

const ID_FILE_ADD: i32 = 1001;
const ID_FILE_REMOVE: i32 = 1002;
const ID_FILE_CLEAR: i32 = 1003;
const ID_OUTPUT_BROWSE: i32 = 1004;
const ID_START: i32 = 1005;
const ID_STOP: i32 = 1006;
const ID_CHECK_UPDATE: i32 = 1007;
const ID_SUPPORT_REVOLUT: i32 = 1008;
const ID_SUPPORT_PAYPAL: i32 = 1009;
const ID_LANGUAGE: i32 = 1010;
const ID_THEME: i32 = 1011;

#[repr(C)]
struct WndClassW {
    style: Uint,
    lpfn_wnd_proc: Option<unsafe extern "system" fn(Hwnd, Uint, Wparam, Lparam) -> Lresult>,
    cb_cls_extra: i32,
    cb_wnd_extra: i32,
    h_instance: Hinstance,
    h_icon: Hicon,
    h_cursor: Hcursor,
    hbr_background: Hbrush,
    lpsz_menu_name: *const u16,
    lpsz_class_name: *const u16,
}

#[repr(C)]
struct Point {
    x: i32,
    y: i32,
}

#[repr(C)]
struct Msg {
    hwnd: Hwnd,
    message: Uint,
    w_param: Wparam,
    l_param: Lparam,
    time: Dword,
    pt: Point,
}

#[repr(C)]
struct OpenFileNameW {
    l_struct_size: Dword,
    hwnd_owner: Hwnd,
    h_instance: Hinstance,
    lpstr_filter: *const u16,
    lpstr_custom_filter: *mut u16,
    n_max_cust_filter: Dword,
    n_filter_index: Dword,
    lpstr_file: *mut u16,
    n_max_file: Dword,
    lpstr_file_title: *mut u16,
    n_max_file_title: Dword,
    lpstr_initial_dir: *const u16,
    lpstr_title: *const u16,
    flags: Dword,
    n_file_offset: u16,
    n_file_extension: u16,
    lpstr_def_ext: *const u16,
    l_cust_data: isize,
    lpfn_hook: *mut core::ffi::c_void,
    lp_template_name: *const u16,
    pv_reserved: *mut core::ffi::c_void,
    dw_reserved: Dword,
    flags_ex: Dword,
}

#[repr(C)]
struct BrowseInfoW {
    hwnd_owner: Hwnd,
    pidl_root: *mut core::ffi::c_void,
    psz_display_name: *mut u16,
    lpsz_title: *const u16,
    ul_flags: Uint,
    lpfn: *mut core::ffi::c_void,
    l_param: Lparam,
    i_image: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Guid {
    data1: u32,
    data2: u16,
    data3: u16,
    data4: [u8; 8],
}

#[repr(C)]
struct IAccPropServices {
    lp_vtbl: *const IAccPropServicesVtbl,
}

#[repr(C)]
struct IAccPropServicesVtbl {
    query_interface: unsafe extern "system" fn(*mut IAccPropServices, *const Guid, *mut *mut core::ffi::c_void) -> Hresult,
    add_ref: unsafe extern "system" fn(*mut IAccPropServices) -> Ulong,
    release: unsafe extern "system" fn(*mut IAccPropServices) -> Ulong,
    set_prop_value: usize,
    set_prop_server: usize,
    clear_props: usize,
    set_hwnd_prop: usize,
    set_hwnd_prop_str: unsafe extern "system" fn(*mut IAccPropServices, Hwnd, Dword, Dword, Guid, *const u16) -> Hresult,
}

type Hresult = i32;
type Ulong = u32;

const CLSID_ACC_PROP_SERVICES: Guid = Guid {
    data1: 0xb5f8350b,
    data2: 0x0548,
    data3: 0x48b1,
    data4: [0xa6, 0xee, 0x88, 0xbd, 0x00, 0xb4, 0xa5, 0xe7],
};

const IID_IACC_PROP_SERVICES: Guid = Guid {
    data1: 0x6e26e776,
    data2: 0x04f0,
    data3: 0x495d,
    data4: [0x80, 0xe4, 0x33, 0x30, 0x35, 0x2e, 0x31, 0x69],
};

const PROPID_ACC_NAME: Guid = Guid {
    data1: 0x608d3df8,
    data2: 0x8128,
    data3: 0x4aa7,
    data4: [0xa4, 0x28, 0xf5, 0x5e, 0x49, 0x26, 0x72, 0x91],
};

const PROPID_ACC_DESCRIPTION: Guid = Guid {
    data1: 0x4d48dfe4,
    data2: 0xbd3f,
    data3: 0x491f,
    data4: [0xa6, 0x48, 0x49, 0x2d, 0x6f, 0x20, 0xc5, 0x88],
};

#[link(name = "user32")]
extern "system" {
    fn RegisterClassW(class: *const WndClassW) -> u16;
    fn CreateWindowExW(
        ex_style: Dword,
        class_name: *const u16,
        window_name: *const u16,
        style: Dword,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        parent: Hwnd,
        menu: Hmenu,
        instance: Hinstance,
        param: *mut core::ffi::c_void,
    ) -> Hwnd;
    fn DefWindowProcW(hwnd: Hwnd, msg: Uint, w_param: Wparam, l_param: Lparam) -> Lresult;
    fn DestroyWindow(hwnd: Hwnd) -> Bool;
    fn DispatchMessageW(msg: *const Msg) -> Lresult;
    fn EnableWindow(hwnd: Hwnd, enable: Bool) -> Bool;
    fn GetClientRect(hwnd: Hwnd, rect: *mut Rect) -> Bool;
    fn GetClassNameW(hwnd: Hwnd, buffer: *mut u16, max_count: i32) -> i32;
    fn GetFocus() -> Hwnd;
    fn GetKeyState(virtual_key: i32) -> i16;
    fn GetMessageW(msg: *mut Msg, hwnd: Hwnd, min_filter: Uint, max_filter: Uint) -> Bool;
    fn GetWindowTextLengthW(hwnd: Hwnd) -> i32;
    fn GetWindowTextW(hwnd: Hwnd, buffer: *mut u16, max_count: i32) -> i32;
    fn IsDialogMessageW(hwnd: Hwnd, msg: *mut Msg) -> Bool;
    fn IsWindowEnabled(hwnd: Hwnd) -> Bool;
    fn IsWindowVisible(hwnd: Hwnd) -> Bool;
    fn LoadCursorW(instance: Hinstance, cursor_name: *const u16) -> Hcursor;
    fn MessageBoxW(hwnd: Hwnd, text: *const u16, caption: *const u16, flags: Uint) -> i32;
    fn MoveWindow(hwnd: Hwnd, x: i32, y: i32, width: i32, height: i32, repaint: Bool) -> Bool;
    fn PostMessageW(hwnd: Hwnd, msg: Uint, w_param: Wparam, l_param: Lparam) -> Bool;
    fn PostQuitMessage(exit_code: i32);
    fn SendMessageW(hwnd: Hwnd, msg: Uint, w_param: Wparam, l_param: Lparam) -> Lresult;
    fn SetFocus(hwnd: Hwnd) -> Hwnd;
    fn SetWindowTextW(hwnd: Hwnd, text: *const u16) -> Bool;
    fn ShowWindow(hwnd: Hwnd, cmd_show: i32) -> Bool;
    fn TranslateMessage(msg: *const Msg) -> Bool;
    fn UpdateWindow(hwnd: Hwnd) -> Bool;
    fn FillRect(hdc: Hdc, rect: *const Rect, brush: Hbrush) -> i32;
    fn InvalidateRect(hwnd: Hwnd, rect: *const Rect, erase: Bool) -> Bool;
    fn NotifyWinEvent(event: Dword, hwnd: Hwnd, object_id: Dword, child_id: i32);
}

#[link(name = "kernel32")]
extern "system" {
    fn CloseHandle(handle: isize) -> Bool;
    fn GetModuleHandleW(module_name: *const u16) -> Hinstance;
    fn OpenProcess(desired_access: Dword, inherit_handle: Bool, process_id: Dword) -> isize;
    fn SetPriorityClass(process: isize, priority_class: Dword) -> Bool;
}

#[link(name = "urlmon")]
extern "system" {
    fn URLDownloadToFileW(
        caller: *mut core::ffi::c_void,
        url: *const u16,
        file_name: *const u16,
        reserved: Dword,
        callback: *mut core::ffi::c_void,
    ) -> Hresult;
}

#[repr(C)]
struct InitCommonControlsEx {
    dw_size: Dword,
    dw_icc: Dword,
}

#[link(name = "comctl32")]
extern "system" {
    fn InitCommonControlsEx(init: *const InitCommonControlsEx) -> Bool;
}

#[link(name = "gdi32")]
extern "system" {
    fn GetStockObject(index: i32) -> isize;
    fn CreateSolidBrush(color: Dword) -> Hbrush;
    fn SetBkColor(hdc: Hdc, color: Dword) -> Dword;
    fn SetBkMode(hdc: Hdc, mode: i32) -> i32;
    fn SetTextColor(hdc: Hdc, color: Dword) -> Dword;
}

#[link(name = "dwmapi")]
extern "system" {
    fn DwmSetWindowAttribute(hwnd: Hwnd, attribute: Dword, value: *const core::ffi::c_void, size: Dword) -> Hresult;
}

#[link(name = "advapi32")]
extern "system" {
    fn RegGetValueW(
        key: Hkey,
        sub_key: *const u16,
        value: *const u16,
        flags: Dword,
        value_type: *mut Dword,
        data: *mut core::ffi::c_void,
        data_size: *mut Dword,
    ) -> i32;
}

#[link(name = "uxtheme")]
extern "system" {
    fn SetWindowTheme(hwnd: Hwnd, sub_app_name: *const u16, sub_id_list: *const u16) -> Hresult;
}

#[link(name = "comdlg32")]
extern "system" {
    fn GetOpenFileNameW(open_file_name: *mut OpenFileNameW) -> Bool;
}

#[link(name = "shell32")]
extern "system" {
    fn SHBrowseForFolderW(info: *mut BrowseInfoW) -> *mut core::ffi::c_void;
    fn SHGetPathFromIDListW(pidl: *mut core::ffi::c_void, path: *mut u16) -> Bool;
    fn ShellExecuteW(
        hwnd: Hwnd,
        operation: *const u16,
        file: *const u16,
        parameters: *const u16,
        directory: *const u16,
        show_command: i32,
    ) -> isize;
}

#[link(name = "ole32")]
extern "system" {
    fn CoCreateInstance(
        class_id: *const Guid,
        outer: *mut core::ffi::c_void,
        class_context: Dword,
        interface_id: *const Guid,
        object: *mut *mut core::ffi::c_void,
    ) -> Hresult;
    fn CoInitializeEx(reserved: *mut core::ffi::c_void, coinit: Dword) -> Hresult;
    fn CoTaskMemFree(pv: *mut core::ffi::c_void);
    fn CoUninitialize();
}

#[repr(C)]
#[derive(Default, Clone, Copy)]
struct Rect {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

#[derive(Default, Clone, Copy)]
struct Controls {
    label_files: Hwnd,
    file_list: Hwnd,
    add_button: Hwnd,
    remove_button: Hwnd,
    clear_button: Hwnd,
    label_output: Hwnd,
    output_edit: Hwnd,
    output_button: Hwnd,
    label_minutes: Hwnd,
    minutes_edit: Hwnd,
    label_format: Hwnd,
    format_combo: Hwnd,
    label_resource: Hwnd,
    resource_combo: Hwnd,
    label_bitrate: Hwnd,
    bitrate_combo: Hwnd,
    label_sample_rate: Hwnd,
    sample_rate_combo: Hwnd,
    smart_check: Hwnd,
    start_button: Hwnd,
    stop_button: Hwnd,
    update_button: Hwnd,
    support_revolut_button: Hwnd,
    support_paypal_button: Hwnd,
    label_language: Hwnd,
    language_combo: Hwnd,
    label_theme: Hwnd,
    theme_combo: Hwnd,
    version_label: Hwnd,
    label_progress: Hwnd,
    progress_bar: Hwnd,
    label_log: Hwnd,
    log_edit: Hwnd,
}

#[derive(Default)]
struct AppState {
    controls: Controls,
    files: Vec<PathBuf>,
    running: bool,
    updating: bool,
    cancel: Option<Arc<AtomicBool>>,
}

#[derive(Clone, Copy)]
enum OutputFormat {
    Same,
    Mp3,
    M4a,
    Wav,
    Flac,
    Opus,
}

#[derive(Clone, Copy)]
enum ResourceProfile {
    Low,
    Normal,
    Fast,
}

struct SplitRequest {
    files: Vec<PathBuf>,
    output_root: Option<PathBuf>,
    segment_seconds: f64,
    format: OutputFormat,
    resource: ResourceProfile,
    bitrate_kbps: u32,
    sample_rate: Option<u32>,
    smart: bool,
    prefer_nvidia: bool,
    ffmpeg: PathBuf,
    ffprobe: PathBuf,
    cancel: Arc<AtomicBool>,
}

#[derive(Clone)]
struct ChapterDetectorConfig {
    cpu_url: String,
    cpu_sha256: String,
    cuda_url: String,
    cuda_sha256: String,
    model_url: String,
    model_sha256: String,
    model_size: u64,
}

struct ChapterDetector {
    cpu_exe: PathBuf,
    primary_exe: PathBuf,
    model: PathBuf,
    using_nvidia: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct SilencePoint {
    start: f64,
    end: f64,
}

impl SilencePoint {
    fn midpoint(self) -> f64 {
        (self.start + self.end) / 2.0
    }

    fn duration(self) -> f64 {
        (self.end - self.start).max(0.0)
    }
}

#[derive(Default)]
struct CompletionSummary {
    input_files_finished: usize,
    output_files_created: usize,
    errors: usize,
    cancelled: bool,
    output_directories: Vec<PathBuf>,
}

enum UpdateOutcome {
    Current,
    Ready {
        version: String,
        changelog: String,
        downloaded_file: PathBuf,
    },
    Error(String),
}

struct UpdateResult {
    outcome: UpdateOutcome,
    manual: bool,
}

static STATE: OnceLock<Mutex<AppState>> = OnceLock::new();
static LANGUAGE: AtomicU8 = AtomicU8::new(Language::Polish as u8);
static DARK_THEME: AtomicBool = AtomicBool::new(false);
static DARK_BACKGROUND_BRUSH: OnceLock<Hbrush> = OnceLock::new();
static DARK_CONTROL_BRUSH: OnceLock<Hbrush> = OnceLock::new();

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Language {
    Polish,
    English,
    German,
    Russian,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ThemePreference {
    System,
    Light,
    Dark,
}

static THEME_PREFERENCE: AtomicU8 = AtomicU8::new(ThemePreference::System as u8);

fn language_from_index(index: u8) -> Option<Language> {
    match index {
        0 => Some(Language::Polish),
        1 => Some(Language::English),
        2 => Some(Language::German),
        3 => Some(Language::Russian),
        _ => None,
    }
}

fn language() -> Language {
    language_from_index(LANGUAGE.load(Ordering::SeqCst)).unwrap_or(Language::Polish)
}

fn theme_from_index(index: u8) -> Option<ThemePreference> {
    match index {
        0 => Some(ThemePreference::System),
        1 => Some(ThemePreference::Light),
        2 => Some(ThemePreference::Dark),
        _ => None,
    }
}

fn theme_preference() -> ThemePreference {
    theme_from_index(THEME_PREFERENCE.load(Ordering::SeqCst))
        .unwrap_or(ThemePreference::System)
}

fn tr(polish: &'static str, english: &'static str, german: &'static str, russian: &'static str) -> &'static str {
    match language() {
        Language::Polish => polish,
        Language::English => english,
        Language::German => german,
        Language::Russian => russian,
    }
}

fn format_tr(
    polish: &'static str,
    english: &'static str,
    german: &'static str,
    russian: &'static str,
    values: &[String],
) -> String {
    let mut result = tr(polish, english, german, russian).to_string();
    for value in values {
        result = result.replacen("{}", value, 1);
    }
    result
}

fn language_code(value: Language) -> &'static str {
    match value {
        Language::Polish => "pl",
        Language::English => "en",
        Language::German => "de",
        Language::Russian => "ru",
    }
}

fn parse_language_setting(contents: &str) -> Option<Language> {
    contents.lines().find_map(|line| {
        let (key, value) = line.split_once('=')?;
        if !key.trim().eq_ignore_ascii_case("language") {
            return None;
        }
        match value.trim().to_ascii_lowercase().as_str() {
            "pl" | "polish" | "polski" => Some(Language::Polish),
            "en" | "english" => Some(Language::English),
            "de" | "german" | "deutsch" => Some(Language::German),
            "ru" | "russian" => Some(Language::Russian),
            _ => None,
        }
    })
}

fn theme_code(value: ThemePreference) -> &'static str {
    match value {
        ThemePreference::System => "system",
        ThemePreference::Light => "light",
        ThemePreference::Dark => "dark",
    }
}

fn parse_theme_setting(contents: &str) -> Option<ThemePreference> {
    contents.lines().find_map(|line| {
        let (key, value) = line.split_once('=')?;
        if !key.trim().eq_ignore_ascii_case("theme") {
            return None;
        }
        match value.trim().to_ascii_lowercase().as_str() {
            "system" => Some(ThemePreference::System),
            "light" => Some(ThemePreference::Light),
            "dark" => Some(ThemePreference::Dark),
            _ => None,
        }
    })
}

fn language_ini_path() -> Result<PathBuf, String> {
    executable_directory().map(|directory| directory.join("Majher.ini"))
}

fn load_language_setting() -> Option<Language> {
    let path = language_ini_path().ok()?;
    let contents = fs::read_to_string(path).ok()?;
    parse_language_setting(&contents)
}

fn load_theme_setting() -> Option<ThemePreference> {
    let path = language_ini_path().ok()?;
    let contents = fs::read_to_string(path).ok()?;
    parse_theme_setting(&contents)
}

fn save_settings(selected_language: Language, selected_theme: ThemePreference) -> Result<(), String> {
    let path = language_ini_path()?;
    fs::write(
        &path,
        format!(
            "[Majher]\r\nlanguage={}\r\ntheme={}\r\n",
            language_code(selected_language),
            theme_code(selected_theme),
        ),
    )
    .map_err(|error| format!("{}: {}", path.display(), error))
}

unsafe fn choose_initial_language() -> Language {
    let title = wide("Majher - Language / Język / Sprache / Язык");
    let first = wide(
        "Choose the interface language / Wybierz język interfejsu / Sprache wählen / Выберите язык.\n\nTak / Yes / Ja / Да: Polski\nNie / No / Nein / Нет: English\nAnuluj / Cancel / Abbrechen / Отмена: Deutsch oder Русский",
    );
    match MessageBoxW(
        0,
        first.as_ptr(),
        title.as_ptr(),
        MB_YESNOCANCEL | MB_ICONINFORMATION,
    ) {
        IDYES => Language::Polish,
        IDNO => Language::English,
        _ => {
            let second = wide(
                "Deutsch oder Русский?\n\nJa / Yes / Да: Deutsch\nNein / No / Нет: Русский",
            );
            if MessageBoxW(
                0,
                second.as_ptr(),
                title.as_ptr(),
                MB_YESNO | MB_ICONINFORMATION,
            ) == IDYES
            {
                Language::German
            } else {
                Language::Russian
            }
        }
    }
}

unsafe fn initialize_language() {
    let loaded = load_language_setting();
    let selected = loaded.unwrap_or_else(|| choose_initial_language());
    let selected_theme = load_theme_setting().unwrap_or(ThemePreference::System);
    LANGUAGE.store(selected as u8, Ordering::SeqCst);
    THEME_PREFERENCE.store(selected_theme as u8, Ordering::SeqCst);
    if loaded.is_none() {
        if let Err(error) = save_settings(selected, selected_theme) {
            show_message(
                0,
                &format_tr(
                    "Nie udało się zapisać ustawienia języka w Majher.ini.\n\n{}",
                    "Could not save the language setting in Majher.ini.\n\n{}",
                    "Die Spracheinstellung konnte nicht in Majher.ini gespeichert werden.\n\n{}",
                    "Не удалось сохранить язык в Majher.ini.\n\n{}",
                    &[error],
                ),
            );
        }
    }
}

fn translate_runtime(text: &str) -> String {
    let pairs: &[(&str, &str)] = match language() {
        Language::Polish => return text.to_string(),
        Language::English => &[
            ("Zatrzymanie zostało zlecone. Kończę bieżący proces FFmpeg lub modelu...", "Stop requested. Finishing the current FFmpeg or model process..."),
            ("Nie udało się", "Could not"),
            ("nie udało się", "could not"),
            ("Nie można", "Cannot"),
            ("nie można", "cannot"),
            ("Błąd przygotowania", "Preparation error"),
            ("Błąd wewnętrzny", "Internal error"),
            ("Błąd", "Error"),
            ("błąd", "error"),
            ("Start programu", "Starting"),
            ("Liczba plików", "Number of files"),
            ("Długość fragmentu", "Segment length"),
            ("Długość pliku", "File duration"),
            ("Folder wyjściowy", "Output folder"),
            ("Liczba fragmentów", "Number of segments"),
            ("Plik zakończony", "File completed"),
            ("Plik zatrzymany przed końcem", "File stopped before completion"),
            ("Przerwano przed kolejnym plikiem", "Stopped before the next file"),
            ("Częstotliwość", "Sample rate"),
            ("bez zmian", "unchanged"),
            ("Sprawdzam aktualizację", "Checking for updates"),
            ("Obecna wersja", "Current version"),
            ("Automatyczna kontrola", "Automatic check"),
            ("jest aktualna", "is up to date"),
            ("Rozpoznawanie będzie działało na CPU", "Recognition will run on the CPU"),
            ("Używam CPU", "Using CPU"),
            ("Pobieram", "Downloading"),
            ("Pobrano", "Downloaded"),
            ("został pobrany i zweryfikowany", "was downloaded and verified"),
            ("program działa, proszę czekać", "the program is working, please wait"),
            ("Znaleziono", "Found"),
            ("Przygotowano", "Prepared"),
            ("Uruchamiam", "Starting"),
            ("Rozpoznano granicę", "Detected boundary"),
            ("Wykryto FFmpeg i FFprobe zainstalowane w systemie.", "FFmpeg and FFprobe were found in the system."),
            ("Majher użyje wersji systemowej i nie będzie niczego pobierać.", "Majher will use the system version and will not download anything."),
            ("FFmpeg nie jest dostępny w systemie. Tworzę katalog tools i pobieram potrzebne pliki...", "FFmpeg is not available in the system. Creating the tools folder and downloading the required files..."),
            ("Pobrano FFmpeg. Rozpakowuję paczkę...", "FFmpeg downloaded. Extracting the package..."),
            ("Przygotowuję lokalne rozpoznawanie rozdziałów audiobooka.", "Preparing local audiobook chapter recognition."),
            ("Silnik CUDA potwierdził aktywne przyspieszenie NVIDIA", "The CUDA engine confirmed active NVIDIA acceleration"),
            ("Silnik CUDA nie uruchomił backendu NVIDIA. Używam CPU", "The CUDA engine did not start the NVIDIA backend. Using CPU"),
            ("Nie udało się przygotować przyspieszenia NVIDIA. Używam CPU", "Could not prepare NVIDIA acceleration. Using CPU"),
            ("Model Whisper został pobrany i zweryfikowany.", "The Whisper model was downloaded and verified."),
            ("Znaleziono rozdziały zapisane w pliku", "Found chapters embedded in the file"),
            ("Używam ich jako najpewniejszych granic", "Using them as the most reliable boundaries"),
            ("Plik nie zawiera zapisanej listy rozdziałów.", "The file contains no embedded chapter list."),
            ("Tryb audiobook: szukam dłuższych pauz, również w nagraniach z szumem taśmy.", "Audiobook mode: looking for longer pauses, including in recordings with tape noise."),
            ("Znalezione dłuższe pauzy", "Longer pauses found"),
            ("Pewne granice audiobooka użyte do cięcia", "Reliable audiobook boundaries used for splitting"),
            ("Nie rozpoznano jednoznacznej zapowiedzi rozdziału. Stosuję bezpieczny podział według długości i pobliskich pauz.", "No clear chapter announcement was recognized. Using safe splitting by length and nearby pauses."),
            ("Brak wystarczająco długich pauz do sprawdzenia zapowiedzi modelem.", "No sufficiently long pauses to check with the model."),
            ("Sprawdzam modelem", "Checking with the model"),
            ("miejsc mogących rozpoczynać nowy rozdział", "positions that may start a new chapter"),
            ("Cięcie bez rekompresji nie powiodło się, próbuję MP3. Szczegóły", "Splitting without recompression failed; trying MP3. Details"),
            ("Skorygowano długość według pełnego skanowania FFmpeg", "Duration corrected using the full FFmpeg scan"),
            ("Plik:", "File:"),
            ("Szczegóły", "Details"),
            ("minut", "minutes"),
        ],
        Language::German => &[
            ("Zatrzymanie zostało zlecone. Kończę bieżący proces FFmpeg lub modelu...", "Stopp angefordert. Der aktuelle FFmpeg- oder Modellprozess wird beendet..."),
            ("Nie udało się", "Nicht möglich"),
            ("nie udało się", "nicht möglich"),
            ("Nie można", "Nicht möglich"),
            ("nie można", "nicht möglich"),
            ("Błąd przygotowania", "Vorbereitungsfehler"),
            ("Błąd wewnętrzny", "Interner Fehler"),
            ("Błąd", "Fehler"),
            ("błąd", "Fehler"),
            ("Start programu", "Programmstart"),
            ("Liczba plików", "Dateianzahl"),
            ("Długość fragmentu", "Segmentlänge"),
            ("Długość pliku", "Dateidauer"),
            ("Folder wyjściowy", "Ausgabeordner"),
            ("Liczba fragmentów", "Anzahl der Segmente"),
            ("Plik zakończony", "Datei abgeschlossen"),
            ("Plik zatrzymany przed końcem", "Datei vor dem Ende gestoppt"),
            ("Przerwano przed kolejnym plikiem", "Vor der nächsten Datei gestoppt"),
            ("Częstotliwość", "Abtastrate"),
            ("bez zmian", "unverändert"),
            ("Sprawdzam aktualizację", "Suche nach Updates"),
            ("Obecna wersja", "Aktuelle Version"),
            ("Automatyczna kontrola", "Automatische Prüfung"),
            ("jest aktualna", "ist aktuell"),
            ("Rozpoznawanie będzie działało na CPU", "Die Erkennung läuft auf der CPU"),
            ("Używam CPU", "CPU wird verwendet"),
            ("Pobieram", "Herunterladen"),
            ("Pobrano", "Heruntergeladen"),
            ("został pobrany i zweryfikowany", "wurde heruntergeladen und geprüft"),
            ("program działa, proszę czekać", "das Programm arbeitet, bitte warten"),
            ("Znaleziono", "Gefunden"),
            ("Przygotowano", "Vorbereitet"),
            ("Uruchamiam", "Start"),
            ("Rozpoznano granicę", "Grenze erkannt"),
            ("Wykryto FFmpeg i FFprobe zainstalowane w systemie.", "FFmpeg und FFprobe wurden im System gefunden."),
            ("Majher użyje wersji systemowej i nie będzie niczego pobierać.", "Majher verwendet die Systemversion und lädt nichts herunter."),
            ("FFmpeg nie jest dostępny w systemie. Tworzę katalog tools i pobieram potrzebne pliki...", "FFmpeg ist im System nicht verfügbar. Der Ordner tools wird erstellt und die benötigten Dateien werden heruntergeladen..."),
            ("Pobrano FFmpeg. Rozpakowuję paczkę...", "FFmpeg wurde heruntergeladen. Das Paket wird entpackt..."),
            ("Przygotowuję lokalne rozpoznawanie rozdziałów audiobooka.", "Lokale Hörbuch-Kapitelerkennung wird vorbereitet."),
            ("Silnik CUDA potwierdził aktywne przyspieszenie NVIDIA", "Die CUDA-Engine hat die aktive NVIDIA-Beschleunigung bestätigt"),
            ("Silnik CUDA nie uruchomił backendu NVIDIA. Używam CPU", "Die CUDA-Engine hat das NVIDIA-Backend nicht gestartet. CPU wird verwendet"),
            ("Nie udało się przygotować przyspieszenia NVIDIA. Używam CPU", "NVIDIA-Beschleunigung konnte nicht vorbereitet werden. CPU wird verwendet"),
            ("Model Whisper został pobrany i zweryfikowany.", "Das Whisper-Modell wurde heruntergeladen und geprüft."),
            ("Znaleziono rozdziały zapisane w pliku", "Eingebettete Kapitel wurden gefunden"),
            ("Używam ich jako najpewniejszych granic", "Sie werden als zuverlässigste Grenzen verwendet"),
            ("Plik nie zawiera zapisanej listy rozdziałów.", "Die Datei enthält keine eingebettete Kapitelliste."),
            ("Tryb audiobook: szukam dłuższych pauz, również w nagraniach z szumem taśmy.", "Hörbuchmodus: Suche nach längeren Pausen, auch in Aufnahmen mit Bandrauschen."),
            ("Znalezione dłuższe pauzy", "Gefundene längere Pausen"),
            ("Pewne granice audiobooka użyte do cięcia", "Zuverlässige Hörbuchgrenzen für die Teilung"),
            ("Nie rozpoznano jednoznacznej zapowiedzi rozdziału. Stosuję bezpieczny podział według długości i pobliskich pauz.", "Keine eindeutige Kapitelansage erkannt. Sichere Teilung nach Länge und nahen Pausen wird verwendet."),
            ("Brak wystarczająco długich pauz do sprawdzenia zapowiedzi modelem.", "Keine ausreichend langen Pausen für die Modellprüfung."),
            ("Sprawdzam modelem", "Prüfung mit dem Modell"),
            ("miejsc mogących rozpoczynać nowy rozdział", "Positionen, die ein neues Kapitel beginnen könnten"),
            ("Cięcie bez rekompresji nie powiodło się, próbuję MP3. Szczegóły", "Teilung ohne Neukodierung fehlgeschlagen; MP3 wird versucht. Details"),
            ("Skorygowano długość według pełnego skanowania FFmpeg", "Dauer anhand des vollständigen FFmpeg-Scans korrigiert"),
            ("Plik:", "Datei:"),
            ("Szczegóły", "Details"),
            ("minut", "Minuten"),
        ],
        Language::Russian => &[
            ("Zatrzymanie zostało zlecone. Kończę bieżący proces FFmpeg lub modelu...", "Запрошена остановка. Завершается текущий процесс FFmpeg или модели..."),
            ("Nie udało się", "Не удалось"),
            ("nie udało się", "не удалось"),
            ("Nie można", "Невозможно"),
            ("nie można", "невозможно"),
            ("Błąd przygotowania", "Ошибка подготовки"),
            ("Błąd wewnętrzny", "Внутренняя ошибка"),
            ("Błąd", "Ошибка"),
            ("błąd", "ошибка"),
            ("Start programu", "Запуск программы"),
            ("Liczba plików", "Количество файлов"),
            ("Długość fragmentu", "Длина фрагмента"),
            ("Długość pliku", "Длительность файла"),
            ("Folder wyjściowy", "Папка вывода"),
            ("Liczba fragmentów", "Количество фрагментов"),
            ("Plik zakończony", "Файл обработан"),
            ("Plik zatrzymany przed końcem", "Обработка файла остановлена"),
            ("Przerwano przed kolejnym plikiem", "Остановлено перед следующим файлом"),
            ("Częstotliwość", "Частота дискретизации"),
            ("bez zmian", "без изменений"),
            ("Sprawdzam aktualizację", "Проверка обновлений"),
            ("Obecna wersja", "Текущая версия"),
            ("Automatyczna kontrola", "Автоматическая проверка"),
            ("jest aktualna", "актуальна"),
            ("Rozpoznawanie będzie działało na CPU", "Распознавание будет работать на CPU"),
            ("Używam CPU", "Используется CPU"),
            ("Pobieram", "Загрузка"),
            ("Pobrano", "Загружено"),
            ("został pobrany i zweryfikowany", "загружен и проверен"),
            ("program działa, proszę czekać", "программа работает, подождите"),
            ("Znaleziono", "Найдено"),
            ("Przygotowano", "Подготовлено"),
            ("Uruchamiam", "Запуск"),
            ("Rozpoznano granicę", "Распознана граница"),
            ("Wykryto FFmpeg i FFprobe zainstalowane w systemie.", "В системе найдены FFmpeg и FFprobe."),
            ("Majher użyje wersji systemowej i nie będzie niczego pobierać.", "Majher будет использовать системную версию и ничего не будет загружать."),
            ("FFmpeg nie jest dostępny w systemie. Tworzę katalog tools i pobieram potrzebne pliki...", "FFmpeg недоступен в системе. Создается папка tools и загружаются необходимые файлы..."),
            ("Pobrano FFmpeg. Rozpakowuję paczkę...", "FFmpeg загружен. Распаковка пакета..."),
            ("Przygotowuję lokalne rozpoznawanie rozdziałów audiobooka.", "Подготовка локального распознавания глав аудиокниги."),
            ("Silnik CUDA potwierdził aktywne przyspieszenie NVIDIA", "Движок CUDA подтвердил активное ускорение NVIDIA"),
            ("Silnik CUDA nie uruchomił backendu NVIDIA. Używam CPU", "Движок CUDA не запустил серверную часть NVIDIA. Используется CPU"),
            ("Nie udało się przygotować przyspieszenia NVIDIA. Używam CPU", "Не удалось подготовить ускорение NVIDIA. Используется CPU"),
            ("Model Whisper został pobrany i zweryfikowany.", "Модель Whisper загружена и проверена."),
            ("Znaleziono rozdziały zapisane w pliku", "Найдены главы, встроенные в файл"),
            ("Używam ich jako najpewniejszych granic", "Они используются как наиболее надежные границы"),
            ("Plik nie zawiera zapisanej listy rozdziałów.", "Файл не содержит встроенного списка глав."),
            ("Tryb audiobook: szukam dłuższych pauz, również w nagraniach z szumem taśmy.", "Режим аудиокниги: поиск длинных пауз, в том числе в записях с шумом ленты."),
            ("Znalezione dłuższe pauzy", "Найдено длинных пауз"),
            ("Pewne granice audiobooka użyte do cięcia", "Надежные границы аудиокниги для разделения"),
            ("Nie rozpoznano jednoznacznej zapowiedzi rozdziału. Stosuję bezpieczny podział według długości i pobliskich pauz.", "Четкое объявление главы не распознано. Используется безопасное разделение по длине и ближайшим паузам."),
            ("Brak wystarczająco długich pauz do sprawdzenia zapowiedzi modelem.", "Нет достаточно длинных пауз для проверки моделью."),
            ("Sprawdzam modelem", "Проверка моделью"),
            ("miejsc mogących rozpoczynać nowy rozdział", "позиций, которые могут начинать новую главу"),
            ("Cięcie bez rekompresji nie powiodło się, próbuję MP3. Szczegóły", "Разделение без перекодирования не удалось; выполняется попытка MP3. Подробности"),
            ("Skorygowano długość według pełnego skanowania FFmpeg", "Длительность скорректирована по полному сканированию FFmpeg"),
            ("Plik:", "Файл:"),
            ("Szczegóły", "Подробности"),
            ("minut", "минут"),
        ],
    };
    let mut result = text.to_string();
    for (source, target) in pairs {
        result = result.replace(source, target);
    }
    result
}

unsafe fn system_uses_dark_theme() -> bool {
    let sub_key = wide("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize");
    let value_name = wide("AppsUseLightTheme");
    let mut value_type = 0u32;
    let mut value = 1u32;
    let mut value_size = std::mem::size_of::<u32>() as u32;
    RegGetValueW(
        HKEY_CURRENT_USER,
        sub_key.as_ptr(),
        value_name.as_ptr(),
        RRF_RT_REG_DWORD,
        &mut value_type,
        &mut value as *mut u32 as *mut core::ffi::c_void,
        &mut value_size,
    ) == 0 && value == 0
}

unsafe fn dark_background_brush() -> Hbrush {
    *DARK_BACKGROUND_BRUSH.get_or_init(|| CreateSolidBrush(DARK_BACKGROUND))
}

unsafe fn dark_control_brush() -> Hbrush {
    *DARK_CONTROL_BRUSH.get_or_init(|| CreateSolidBrush(DARK_CONTROL_BACKGROUND))
}

fn theme_controls(controls: Controls) -> Vec<Hwnd> {
    vec![
        controls.label_files,
        controls.file_list,
        controls.add_button,
        controls.remove_button,
        controls.clear_button,
        controls.label_output,
        controls.output_edit,
        controls.output_button,
        controls.label_minutes,
        controls.minutes_edit,
        controls.label_format,
        controls.format_combo,
        controls.label_resource,
        controls.resource_combo,
        controls.label_bitrate,
        controls.bitrate_combo,
        controls.label_sample_rate,
        controls.sample_rate_combo,
        controls.smart_check,
        controls.start_button,
        controls.stop_button,
        controls.update_button,
        controls.support_revolut_button,
        controls.support_paypal_button,
        controls.label_language,
        controls.language_combo,
        controls.label_theme,
        controls.theme_combo,
        controls.version_label,
        controls.label_progress,
        controls.progress_bar,
        controls.label_log,
        controls.log_edit,
    ]
}

fn dark_theme_for(preference: ThemePreference, system_dark: bool) -> bool {
    match preference {
        ThemePreference::System => system_dark,
        ThemePreference::Light => false,
        ThemePreference::Dark => true,
    }
}

unsafe fn apply_system_theme(hwnd: Hwnd, controls: Controls) {
    let dark = dark_theme_for(theme_preference(), system_uses_dark_theme());
    DARK_THEME.store(dark, Ordering::SeqCst);
    let enabled: i32 = if dark { 1 } else { 0 };
    let value = &enabled as *const i32 as *const core::ffi::c_void;
    if DwmSetWindowAttribute(
        hwnd,
        DWMWA_USE_IMMERSIVE_DARK_MODE,
        value,
        std::mem::size_of::<i32>() as u32,
    ) < 0
    {
        DwmSetWindowAttribute(
            hwnd,
            DWMWA_USE_IMMERSIVE_DARK_MODE_BEFORE_20H1,
            value,
            std::mem::size_of::<i32>() as u32,
        );
    }
    let dark_theme = wide("DarkMode_Explorer");
    let classic_theme = wide("");
    for control in theme_controls(controls) {
        if control == 0 {
            continue;
        }
        if dark {
            if is_input_control(control) || control == controls.file_list {
                SetWindowTheme(control, classic_theme.as_ptr(), classic_theme.as_ptr());
            } else {
                SetWindowTheme(control, dark_theme.as_ptr(), null());
            }
        } else {
            SetWindowTheme(control, null(), null());
        }
        InvalidateRect(control, null(), 1);
    }
    InvalidateRect(hwnd, null(), 1);
}

unsafe fn is_input_control(hwnd: Hwnd) -> bool {
    let mut name = [0u16; 32];
    let count = GetClassNameW(hwnd, name.as_mut_ptr(), name.len() as i32);
    if count <= 0 {
        return false;
    }
    let class = wide_to_string(&name[..count as usize]);
    class.eq_ignore_ascii_case("EDIT") || class.eq_ignore_ascii_case("COMBOBOX")
}

unsafe fn draw_dark_control(hdc: Hdc, control_background: bool) -> Lresult {
    SetTextColor(hdc, DARK_TEXT);
    let background = if control_background {
        DARK_CONTROL_BACKGROUND
    } else {
        DARK_BACKGROUND
    };
    SetBkColor(hdc, background);
    SetBkMode(hdc, TRANSPARENT);
    if control_background {
        dark_control_brush()
    } else {
        dark_background_brush()
    }
}

fn main() {
    cleanup_previous_update_backup();
    STATE.set(Mutex::new(AppState::default())).ok();
    unsafe {
        let com_initialized = CoInitializeEx(null_mut(), COINIT_APARTMENTTHREADED) >= 0;
        initialize_language();
        let common_controls = InitCommonControlsEx {
            dw_size: std::mem::size_of::<InitCommonControlsEx>() as Dword,
            dw_icc: ICC_PROGRESS_CLASS,
        };
        InitCommonControlsEx(&common_controls);
        let instance = GetModuleHandleW(null());
        let class_name = wide("MajherWindow");
        let wnd_class = WndClassW {
            style: 0,
            lpfn_wnd_proc: Some(window_proc),
            cb_cls_extra: 0,
            cb_wnd_extra: 0,
            h_instance: instance,
            h_icon: 0,
            h_cursor: LoadCursorW(0, IDC_ARROW as *const u16),
            hbr_background: COLOR_WINDOW + 1,
            lpsz_menu_name: null(),
            lpsz_class_name: class_name.as_ptr(),
        };

        if RegisterClassW(&wnd_class) == 0 {
            show_message(0, tr("Nie można zarejestrować okna programu.", "Could not register the program window.", "Das Programmfenster konnte nicht registriert werden.", "Не удалось зарегистрировать окно программы."));
            return;
        }

        let title = wide(&app_window_title());
        let hwnd = CreateWindowExW(
            WS_EX_CONTROLPARENT,
            class_name.as_ptr(),
            title.as_ptr(),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            960,
            920,
            0,
            0,
            instance,
            null_mut(),
        );

        if hwnd == 0 {
            show_message(0, tr("Nie można utworzyć okna programu.", "Could not create the program window.", "Das Programmfenster konnte nicht erstellt werden.", "Не удалось создать окно программы."));
            return;
        }

        ShowWindow(hwnd, SW_SHOW);
        UpdateWindow(hwnd);

        let mut msg = Msg {
            hwnd: 0,
            message: 0,
            w_param: 0,
            l_param: 0,
            time: 0,
            pt: Point { x: 0, y: 0 },
        };

        while GetMessageW(&mut msg, 0, 0, 0) > 0 {
            if msg.message == WM_KEYDOWN
                && msg.w_param == VK_TAB
                && focus_next_tab_control(GetKeyState(VK_SHIFT) < 0)
            {
                continue;
            }
            if IsDialogMessageW(hwnd, &mut msg) == 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }

        if com_initialized {
            CoUninitialize();
        }
    }
}

unsafe extern "system" fn window_proc(hwnd: Hwnd, msg: Uint, w_param: Wparam, l_param: Lparam) -> Lresult {
    match msg {
        WM_CREATE => {
            let controls = create_controls(hwnd);
            apply_accessible_names(hwnd, controls);
            if let Some(state) = STATE.get() {
                state.lock().unwrap().controls = controls;
            }
            layout_controls(hwnd, controls);
            apply_system_theme(hwnd, controls);
            set_controls_running(controls, false);
            PostMessageW(hwnd, WM_APP_FOCUS_FIRST, 0, 0);
            handle_check_update(hwnd, false);
            0
        }
        WM_APP_FOCUS_FIRST => {
            focus_primary_control();
            0
        }
        WM_SIZE => {
            if let Some(state) = STATE.get() {
                let controls = state.lock().unwrap().controls;
                layout_controls(hwnd, controls);
            }
            0
        }
        WM_SETFOCUS => {
            if GetFocus() == hwnd {
                PostMessageW(hwnd, WM_APP_FOCUS_FIRST, 0, 0);
            }
            0
        }
        WM_SETTINGCHANGE | WM_THEMECHANGED => {
            if let Some(state) = STATE.get() {
                let controls = state.lock().unwrap().controls;
                apply_system_theme(hwnd, controls);
            }
            0
        }
        WM_ERASEBKGND => {
            if DARK_THEME.load(Ordering::SeqCst) {
                let mut rect = Rect::default();
                GetClientRect(hwnd, &mut rect);
                FillRect(w_param as Hdc, &rect, dark_background_brush());
                1
            } else {
                DefWindowProcW(hwnd, msg, w_param, l_param)
            }
        }
        WM_CTLCOLOREDIT | WM_CTLCOLORLISTBOX => {
            if DARK_THEME.load(Ordering::SeqCst) {
                draw_dark_control(w_param as Hdc, true)
            } else {
                DefWindowProcW(hwnd, msg, w_param, l_param)
            }
        }
        WM_CTLCOLORSTATIC | WM_CTLCOLORBTN => {
            if DARK_THEME.load(Ordering::SeqCst) {
                draw_dark_control(w_param as Hdc, is_input_control(l_param as Hwnd))
            } else {
                DefWindowProcW(hwnd, msg, w_param, l_param)
            }
        }
        WM_COMMAND => {
            let id = (w_param & 0xffff) as i32;
            let notification = w_param >> 16;
            match id {
                ID_LANGUAGE if notification == CBN_SELCHANGE => handle_language_change(hwnd),
                ID_THEME if notification == CBN_SELCHANGE => handle_theme_change(hwnd),
                ID_FILE_ADD => handle_add_files(hwnd),
                ID_FILE_REMOVE => handle_remove_file(),
                ID_FILE_CLEAR => handle_clear_files(),
                ID_OUTPUT_BROWSE => handle_browse_output(hwnd),
                ID_START => handle_start(hwnd),
                ID_STOP => handle_stop(),
                ID_CHECK_UPDATE => handle_check_update(hwnd, true),
                ID_SUPPORT_REVOLUT => open_support_url(hwnd, "https://revolut.me/r_rosiejka"),
                ID_SUPPORT_PAYPAL => open_support_url(hwnd, "https://paypal.me/proszczok"),
                _ => {}
            }
            0
        }
        WM_APP_LOG => {
            if l_param != 0 {
                let text = *Box::from_raw(l_param as *mut String);
                if let Some(state) = STATE.get() {
                    let controls = state.lock().unwrap().controls;
                    append_log(controls.log_edit, &text);
                }
            }
            0
        }
        WM_APP_PROGRESS => {
            if let Some(state) = STATE.get() {
                let controls = state.lock().unwrap().controls;
                set_progress(controls.progress_bar, w_param.min(1000) as u32);
            }
            0
        }
        WM_APP_NOTICE => {
            if l_param != 0 {
                let text = *Box::from_raw(l_param as *mut String);
                show_message(hwnd, &text);
                focus_primary_control();
            }
            0
        }
        WM_APP_DONE => {
            let summary = if l_param != 0 {
                Some(*Box::from_raw(l_param as *mut CompletionSummary))
            } else {
                None
            };
            if let Some(state) = STATE.get() {
                let mut state = state.lock().unwrap();
                state.running = false;
                state.cancel = None;
                set_controls_running(state.controls, false);
                if summary.as_ref().is_some_and(|value| value.errors == 0 && !value.cancelled) {
                    set_progress(state.controls.progress_bar, 1000);
                }
                SetFocus(state.controls.start_button);
            }
            if let Some(summary) = summary {
                if summary.errors == 0 && !summary.cancelled {
                    show_success_and_offer_folders(
                        hwnd,
                        &summary,
                    );
                } else if summary.errors > 0 && !summary.cancelled {
                    show_message(
                        hwnd,
                        &format_tr(
                            "Zakończono z błędami.\n\nPoprawnie pocięte pliki wejściowe: {}.\nUtworzone pliki wynikowe: {}.\nBłędy: {}.",
                            "Finished with errors.\n\nInput files completed: {}.\nOutput files created: {}.\nErrors: {}.",
                            "Mit Fehlern beendet.\n\nAbgeschlossene Eingabedateien: {}.\nErstellte Ausgabedateien: {}.\nFehler: {}.",
                            "Завершено с ошибками.\n\nОбработано исходных файлов: {}.\nСоздано файлов: {}.\nОшибок: {}.",
                            &[
                                summary.input_files_finished.to_string(),
                                summary.output_files_created.to_string(),
                                summary.errors.to_string(),
                            ],
                        ),
                    );
                }
            }
            0
        }
        WM_APP_UPDATE_RESULT => {
            if l_param != 0 {
                let result = *Box::from_raw(l_param as *mut UpdateResult);
                if let Some(state) = STATE.get() {
                    let mut state = state.lock().unwrap();
                    state.updating = false;
                    EnableWindow(state.controls.update_button, if state.running { 0 } else { 1 });
                }
                match result.outcome {
                    UpdateOutcome::Current => {
                        if result.manual {
                            show_message(
                                hwnd,
                                &format_tr("Masz najnowszą wersję programu: {}.", "You have the latest version: {}.", "Sie verwenden die neueste Version: {}.", "У вас установлена последняя версия: {}.", &[APP_VERSION.to_string()]),
                            );
                        } else if let Some(state) = STATE.get() {
                            let controls = state.lock().unwrap().controls;
                            append_log(
                                controls.log_edit,
                                &format_tr("Automatyczna kontrola: wersja {} jest aktualna.", "Automatic check: version {} is up to date.", "Automatische Prüfung: Version {} ist aktuell.", "Автоматическая проверка: версия {} актуальна.", &[APP_VERSION.to_string()]),
                            );
                        }
                    }
                    UpdateOutcome::Error(error) => {
                        if result.manual {
                            show_message(hwnd, &format_tr("Nie udało się sprawdzić aktualizacji.\n\n{}", "Could not check for updates.\n\n{}", "Die Updateprüfung ist fehlgeschlagen.\n\n{}", "Не удалось проверить обновления.\n\n{}", &[error]));
                        } else if let Some(state) = STATE.get() {
                            let controls = state.lock().unwrap().controls;
                            append_log(
                                controls.log_edit,
                                &format_tr("Automatyczna kontrola aktualizacji nie powiodła się: {}", "Automatic update check failed: {}", "Die automatische Updateprüfung ist fehlgeschlagen: {}", "Автоматическая проверка обновлений не выполнена: {}", &[error]),
                            );
                        }
                    }
                    UpdateOutcome::Ready {
                        version,
                        changelog,
                        downloaded_file,
                    } => {
                        let changelog: String = changelog.chars().take(1600).collect();
                        let changes = if changelog.trim().is_empty() {
                            tr("Brak opisu zmian.", "No changelog available.", "Keine Änderungsbeschreibung verfügbar.", "Описание изменений отсутствует.")
                        } else {
                            changelog.trim()
                        };
                        let message = wide(&format_tr(
                            "Znaleziono wersję {}. Aktualizacja została pobrana.\n\nCo nowego:\n{}\n\nCzy zamknąć Majhera, podmienić plik programu i uruchomić nową wersję?",
                            "Version {} was found and downloaded.\n\nWhat's new:\n{}\n\nClose Majher, replace the program and start the new version?",
                            "Version {} wurde gefunden und heruntergeladen.\n\nNeuigkeiten:\n{}\n\nMajher schließen, das Programm ersetzen und die neue Version starten?",
                            "Версия {} найдена и загружена.\n\nЧто нового:\n{}\n\nЗакрыть Majher, заменить программу и запустить новую версию?",
                            &[version, changes.to_string()],
                        ));
                        let title = wide(&app_window_title());
                        let answer = MessageBoxW(
                            hwnd,
                            message.as_ptr(),
                            title.as_ptr(),
                            MB_YESNO | MB_ICONINFORMATION,
                        );
                        if answer == IDYES {
                            match launch_update_replacement(&downloaded_file) {
                                Ok(()) => {
                                    DestroyWindow(hwnd);
                                }
                                Err(error) => {
                                    show_message(
                                        hwnd,
                                        &format_tr("Nie udało się uruchomić instalowania aktualizacji.\n\n{}", "Could not start installing the update.\n\n{}", "Die Installation des Updates konnte nicht gestartet werden.\n\n{}", "Не удалось запустить установку обновления.\n\n{}", &[error]),
                                    );
                                }
                            };
                        } else {
                            let _ = fs::remove_file(downloaded_file);
                        }
                    }
                }
            }
            0
        }
        WM_CLOSE => {
            let running = STATE
                .get()
                .map(|s| s.lock().unwrap().running)
                .unwrap_or(false);
            if running {
                show_message(hwnd, tr("Trwa przetwarzanie. Najpierw użyj przycisku zatrzymania i poczekaj na koniec bieżącego fragmentu.", "Processing is in progress. Use Stop now and wait for the current segment to finish.", "Die Verarbeitung läuft. Verwenden Sie Jetzt stoppen und warten Sie auf das Ende des aktuellen Segments.", "Идет обработка. Нажмите Остановить и дождитесь завершения текущего фрагмента."));
                0
            } else {
                DestroyWindow(hwnd);
                0
            }
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, w_param, l_param),
    }
}

unsafe fn create_controls(parent: Hwnd) -> Controls {
    let version_text = format_tr(
        "Wersja {}",
        "Version {}",
        "Version {}",
        "Версия {}",
        &[APP_VERSION.to_string()],
    );
    let controls = Controls {
        label_files: child(parent, "STATIC", tr("Pliki audio do pocięcia", "Audio files to split", "Audiodateien teilen", "Аудиофайлы для разделения"), 0, 0, 0),
        file_list: child(parent, "LISTBOX", "", WS_BORDER | WS_VSCROLL | LBS_NOTIFY | WS_TABSTOP, 0, 0),
        add_button: child(parent, "BUTTON", tr("Dodaj pliki", "Add files", "Dateien hinzufügen", "Добавить файлы"), WS_TABSTOP, 0, ID_FILE_ADD),
        remove_button: child(parent, "BUTTON", tr("Usuń zaznaczony", "Remove selected", "Auswahl entfernen", "Удалить выбранное"), WS_TABSTOP, 0, ID_FILE_REMOVE),
        clear_button: child(parent, "BUTTON", tr("Wyczyść listę", "Clear list", "Liste leeren", "Очистить список"), WS_TABSTOP, 0, ID_FILE_CLEAR),
        label_output: child(parent, "STATIC", tr("Folder wyjściowy, puste pole oznacza folder obok pliku wejściowego", "Output folder; leave empty to save next to the input file", "Ausgabeordner; leer bedeutet neben der Eingabedatei", "Папка вывода; оставьте пустой для сохранения рядом с исходным файлом"), 0, 0, 0),
        output_edit: child(parent, "EDIT", "", WS_BORDER | ES_AUTOHSCROLL | WS_TABSTOP, 0, 0),
        output_button: child(parent, "BUTTON", tr("Wybierz folder", "Choose folder", "Ordner wählen", "Выбрать папку"), WS_TABSTOP, 0, ID_OUTPUT_BROWSE),
        label_minutes: child(parent, "STATIC", tr("Długość jednego fragmentu w minutach", "Segment length in minutes", "Segmentlänge in Minuten", "Длина фрагмента в минутах"), 0, 0, 0),
        minutes_edit: child(parent, "EDIT", "30", WS_BORDER | ES_NUMBER | ES_AUTOHSCROLL | WS_TABSTOP, 0, 0),
        label_format: child(parent, "STATIC", tr("Format wyjściowy", "Output format", "Ausgabeformat", "Формат вывода"), 0, 0, 0),
        format_combo: child(parent, "COMBOBOX", "", CBS_DROPDOWNLIST | CBS_HASSTRINGS | WS_TABSTOP | WS_VSCROLL, 0, 0),
        label_resource: child(parent, "STATIC", tr("Zużycie zasobów komputera", "Computer resource usage", "Ressourcennutzung", "Использование ресурсов"), 0, 0, 0),
        resource_combo: child(parent, "COMBOBOX", "", CBS_DROPDOWNLIST | CBS_HASSTRINGS | WS_TABSTOP | WS_VSCROLL, 0, 0),
        label_bitrate: child(parent, "STATIC", tr("Bitrate pliku wynikowego", "Output bitrate", "Ausgabe-Bitrate", "Битрейт результата"), 0, 0, 0),
        bitrate_combo: child(parent, "COMBOBOX", "", CBS_DROPDOWNLIST | CBS_HASSTRINGS | WS_TABSTOP | WS_VSCROLL, 0, 0),
        label_sample_rate: child(parent, "STATIC", tr("Częstotliwość próbkowania", "Sample rate", "Abtastrate", "Частота дискретизации"), 0, 0, 0),
        sample_rate_combo: child(parent, "COMBOBOX", "", CBS_DROPDOWNLIST | CBS_HASSTRINGS | WS_TABSTOP | WS_VSCROLL, 0, 0),
        smart_check: child(
            parent,
            "BUTTON",
            tr(
                "Tryb audiobook: wykrywaj rozdziały, części, kasety, strony i ścieżki",
                "Audiobook mode: detect chapters, parts, tapes, sides and tracks",
                "Hörbuchmodus: Kapitel, Teile, Kassetten, Seiten und Spuren erkennen",
                "Режим аудиокниги: распознавать главы, части, кассеты, стороны и дорожки",
            ),
            BS_AUTOCHECKBOX | WS_TABSTOP,
            0,
            0,
        ),
        start_button: child(parent, "BUTTON", tr("Uruchom cięcie", "Start splitting", "Teilung starten", "Начать разделение"), WS_TABSTOP, 0, ID_START),
        stop_button: child(parent, "BUTTON", tr("Zatrzymaj teraz", "Stop now", "Jetzt stoppen", "Остановить"), WS_TABSTOP, 0, ID_STOP),
        update_button: child(parent, "BUTTON", tr("Sprawdź aktualizacje", "Check for updates", "Nach Updates suchen", "Проверить обновления"), WS_TABSTOP, 0, ID_CHECK_UPDATE),
        support_revolut_button: child(parent, "BUTTON", tr("Wsparcie: Revolut", "Support: Revolut", "Unterstützen: Revolut", "Поддержать: Revolut"), WS_TABSTOP, 0, ID_SUPPORT_REVOLUT),
        support_paypal_button: child(parent, "BUTTON", tr("Wsparcie: PayPal", "Support: PayPal", "Unterstützen: PayPal", "Поддержать: PayPal"), WS_TABSTOP, 0, ID_SUPPORT_PAYPAL),
        label_language: child(parent, "STATIC", tr("Język interfejsu", "Interface language", "Sprache der Oberfläche", "Язык интерфейса"), 0, 0, 0),
        language_combo: child(parent, "COMBOBOX", "", CBS_DROPDOWNLIST | CBS_HASSTRINGS | WS_TABSTOP | WS_VSCROLL, 0, ID_LANGUAGE),
        label_theme: child(parent, "STATIC", tr("Motyw", "Theme", "Design", "Тема"), 0, 0, 0),
        theme_combo: child(parent, "COMBOBOX", "", CBS_DROPDOWNLIST | CBS_HASSTRINGS | WS_TABSTOP | WS_VSCROLL, 0, ID_THEME),
        version_label: child(parent, "STATIC", &version_text, 0, 0, 0),
        label_progress: child(parent, "STATIC", tr("Postęp przetwarzania", "Processing progress", "Verarbeitungsfortschritt", "Ход обработки"), 0, 0, 0),
        progress_bar: child(parent, "msctls_progress32", "", PBS_SMOOTH | WS_TABSTOP, 0, 0),
        label_log: child(parent, "STATIC", tr("Status i dziennik pracy", "Status and activity log", "Status und Arbeitsprotokoll", "Состояние и журнал работы"), 0, 0, 0),
        log_edit: child(
            parent,
            "EDIT",
            "",
            WS_BORDER | WS_VSCROLL | WS_HSCROLL | ES_MULTILINE | ES_AUTOVSCROLL | ES_READONLY | WS_TABSTOP,
            0,
            0,
        ),
    };

    add_combo_item(controls.format_combo, tr("Jak plik wejściowy, bez rekompresji", "Same as input, no recompression", "Wie Eingabe, ohne Neukodierung", "Как исходный, без перекодирования"));
    add_combo_item(controls.format_combo, "MP3");
    add_combo_item(controls.format_combo, "M4A AAC");
    add_combo_item(controls.format_combo, "WAV");
    add_combo_item(controls.format_combo, "FLAC");
    add_combo_item(controls.format_combo, "OPUS");
    SendMessageW(controls.format_combo, CB_SETCURSEL, 0, 0);

    add_combo_item(controls.resource_combo, tr("Niskie, wolniej i lżej", "Low, slower and lighter", "Niedrig, langsamer und sparsamer", "Низкое, медленнее и легче"));
    add_combo_item(controls.resource_combo, tr("Normalne", "Normal", "Normal", "Обычное"));
    add_combo_item(controls.resource_combo, tr("Szybkie, więcej CPU", "Fast, more CPU", "Schnell, mehr CPU", "Быстрое, больше CPU"));
    SendMessageW(controls.resource_combo, CB_SETCURSEL, 1, 0);

    for bitrate in ["64 kb/s", "96 kb/s", "128 kb/s", "160 kb/s", "192 kb/s", "256 kb/s", "320 kb/s"] {
        add_combo_item(controls.bitrate_combo, bitrate);
    }
    SendMessageW(controls.bitrate_combo, CB_SETCURSEL, 2, 0);

    add_combo_item(controls.sample_rate_combo, tr("Bez zmian, jak plik źródłowy", "Unchanged, same as source", "Unverändert, wie Quelldatei", "Без изменений, как в исходном файле"));
    add_combo_item(controls.sample_rate_combo, "44 100 Hz");
    add_combo_item(controls.sample_rate_combo, "48 000 Hz");
    SendMessageW(controls.sample_rate_combo, CB_SETCURSEL, 0, 0);

    for name in ["Polski", "English", "Deutsch", "Русский"] {
        add_combo_item(controls.language_combo, name);
    }
    SendMessageW(controls.language_combo, CB_SETCURSEL, language() as Wparam, 0);
    populate_theme_combo(controls.theme_combo);

    SendMessageW(controls.smart_check, BM_SETCHECK, BST_CHECKED as Wparam, 0);
    SendMessageW(controls.progress_bar, PBM_SETRANGE32, 0, 1000);
    SendMessageW(controls.progress_bar, PBM_SETPOS, 0, 0);
    controls
}

unsafe fn child(parent: Hwnd, class_name: &str, text: &str, style: Dword, ex_style: Dword, id: i32) -> Hwnd {
    let class_name = wide(class_name);
    let text = wide(text);
    let hwnd = CreateWindowExW(
        ex_style,
        class_name.as_ptr(),
        text.as_ptr(),
        WS_CHILD | WS_VISIBLE | style,
        0,
        0,
        10,
        10,
        parent,
        id as Hmenu,
        GetModuleHandleW(null()),
        null_mut(),
    );
    let font = GetStockObject(DEFAULT_GUI_FONT);
    if hwnd != 0 && font != 0 {
        SendMessageW(hwnd, WM_SETFONT, font as Wparam, 1);
    }
    hwnd
}

unsafe fn layout_controls(parent: Hwnd, controls: Controls) {
    let mut rect = Rect::default();
    if GetClientRect(parent, &mut rect) == 0 {
        return;
    }
    let width = (rect.right - rect.left).max(640);
    let height = (rect.bottom - rect.top).max(810);
    let margin = 14;
    let content = width - margin * 2;
    let button_h = 28;
    let label_h = 20;

    MoveWindow(controls.label_files, margin, 12, content, label_h, 1);
    MoveWindow(controls.file_list, margin, 36, content, 120, 1);
    MoveWindow(controls.add_button, margin, 164, 120, button_h, 1);
    MoveWindow(controls.remove_button, margin + 130, 164, 150, button_h, 1);
    MoveWindow(controls.clear_button, margin + 290, 164, 130, button_h, 1);

    MoveWindow(controls.label_output, margin, 204, content, label_h, 1);
    MoveWindow(controls.output_edit, margin, 228, content - 150, 26, 1);
    MoveWindow(controls.output_button, width - margin - 140, 227, 140, button_h, 1);

    MoveWindow(controls.label_minutes, margin, 266, 260, label_h, 1);
    MoveWindow(controls.minutes_edit, margin, 290, 100, 26, 1);
    MoveWindow(controls.label_format, margin + 130, 266, 170, label_h, 1);
    MoveWindow(controls.format_combo, margin + 130, 290, 240, 180, 1);
    MoveWindow(controls.label_resource, margin + 390, 266, 230, label_h, 1);
    MoveWindow(controls.resource_combo, margin + 390, 290, 230, 180, 1);

    MoveWindow(controls.label_bitrate, margin, 330, 240, label_h, 1);
    MoveWindow(controls.bitrate_combo, margin, 354, 240, 180, 1);
    MoveWindow(controls.label_sample_rate, margin + 270, 330, 280, label_h, 1);
    MoveWindow(controls.sample_rate_combo, margin + 270, 354, 280, 120, 1);
    MoveWindow(controls.smart_check, margin, 390, content, button_h, 1);
    MoveWindow(controls.start_button, margin, 428, 150, button_h + 4, 1);
    MoveWindow(controls.stop_button, margin + 164, 428, 180, button_h + 4, 1);
    MoveWindow(controls.update_button, margin + 358, 428, 180, button_h + 4, 1);
    MoveWindow(controls.version_label, margin + 552, 434, 130, label_h, 1);

    MoveWindow(controls.support_revolut_button, margin, 474, 210, button_h, 1);
    MoveWindow(controls.support_paypal_button, margin + 224, 474, 210, button_h, 1);

    MoveWindow(controls.label_language, margin, 514, 220, label_h, 1);
    MoveWindow(controls.language_combo, margin, 538, 220, 140, 1);
    MoveWindow(controls.label_theme, margin + 244, 514, 220, label_h, 1);
    MoveWindow(controls.theme_combo, margin + 244, 538, 250, 140, 1);

    MoveWindow(controls.label_progress, margin, 586, content, label_h, 1);
    MoveWindow(controls.progress_bar, margin, 610, content, 24, 1);
    MoveWindow(controls.label_log, margin, 646, content, label_h, 1);
    MoveWindow(controls.log_edit, margin, 670, content, (height - 684).max(70), 1);
}

unsafe fn set_controls_running(controls: Controls, running: bool) {
    let enabled = if running { 0 } else { 1 };
    for hwnd in [
        controls.file_list,
        controls.add_button,
        controls.remove_button,
        controls.clear_button,
        controls.output_edit,
        controls.output_button,
        controls.minutes_edit,
        controls.format_combo,
        controls.resource_combo,
        controls.bitrate_combo,
        controls.sample_rate_combo,
        controls.smart_check,
        controls.start_button,
        controls.update_button,
        controls.language_combo,
        controls.theme_combo,
    ] {
        EnableWindow(hwnd, enabled);
    }
    EnableWindow(controls.stop_button, if running { 1 } else { 0 });
}

unsafe fn focus_primary_control() {
    if let Some(state) = STATE.get() {
        let state = state.lock().unwrap();
        let target = if state.running {
            state.controls.stop_button
        } else {
            state.controls.add_button
        };
        if target != 0 {
            SetFocus(target);
        }
    }
}

unsafe fn focus_next_tab_control(backwards: bool) -> bool {
    let Some(state) = STATE.get() else {
        return false;
    };
    let controls = state.lock().unwrap().controls;
    let order = [
        controls.add_button,
        controls.file_list,
        controls.remove_button,
        controls.clear_button,
        controls.output_edit,
        controls.output_button,
        controls.minutes_edit,
        controls.format_combo,
        controls.resource_combo,
        controls.bitrate_combo,
        controls.sample_rate_combo,
        controls.smart_check,
        controls.start_button,
        controls.stop_button,
        controls.update_button,
        controls.support_revolut_button,
        controls.support_paypal_button,
        controls.language_combo,
        controls.theme_combo,
        controls.progress_bar,
        controls.log_edit,
    ];
    let available: Vec<Hwnd> = order
        .into_iter()
        .filter(|hwnd| *hwnd != 0 && IsWindowEnabled(*hwnd) != 0 && IsWindowVisible(*hwnd) != 0)
        .collect();
    if available.is_empty() {
        return false;
    }

    let current = GetFocus();
    let target = if let Some(index) = available.iter().position(|hwnd| *hwnd == current) {
        if backwards {
            available[(index + available.len() - 1) % available.len()]
        } else {
            available[(index + 1) % available.len()]
        }
    } else if backwards {
        *available.last().unwrap()
    } else {
        available[0]
    };
    SetFocus(target);
    true
}

unsafe fn apply_accessible_names(parent: Hwnd, controls: Controls) {
    set_accessible_name(parent, &app_window_title());
    set_accessible_description(
        parent,
        tr(
            "Program do cięcia długich plików audio na krótsze fragmenty.",
            "A program for splitting long audio files into shorter segments.",
            "Ein Programm zum Aufteilen langer Audiodateien in kürzere Segmente.",
            "Программа для разделения длинных аудиофайлов на короткие фрагменты.",
        ),
    );

    set_accessible_name(controls.file_list, tr("Lista plików audio do pocięcia", "Audio files to split", "Liste der zu teilenden Audiodateien", "Список аудиофайлов для разделения"));
    set_accessible_description(
        controls.file_list,
        tr(
            "Lista wybranych plików audio. Użyj przycisku Dodaj pliki, aby dopisać kolejne pozycje.",
            "List of selected audio files. Use Add files to add more items.",
            "Liste der ausgewählten Audiodateien. Mit Dateien hinzufügen können weitere Einträge ergänzt werden.",
            "Список выбранных аудиофайлов. Нажмите Добавить файлы, чтобы добавить другие элементы.",
        ),
    );

    set_accessible_name(controls.add_button, tr("Dodaj pliki", "Add files", "Dateien hinzufügen", "Добавить файлы"));
    set_accessible_name(controls.remove_button, tr("Usuń zaznaczony plik", "Remove selected file", "Ausgewählte Datei entfernen", "Удалить выбранный файл"));
    set_accessible_name(controls.clear_button, tr("Wyczyść listę plików", "Clear file list", "Dateiliste leeren", "Очистить список файлов"));

    set_accessible_name(controls.output_edit, tr("Folder wyjściowy", "Output folder", "Ausgabeordner", "Папка вывода"));
    set_accessible_description(
        controls.output_edit,
        tr(
            "Puste pole oznacza, że program utworzy folder z wynikami obok pliku wejściowego.",
            "Leave empty to create the results folder next to the input file.",
            "Leer lassen, um den Ergebnisordner neben der Eingabedatei zu erstellen.",
            "Оставьте поле пустым, чтобы создать папку результатов рядом с исходным файлом.",
        ),
    );
    set_accessible_name(controls.output_button, tr("Wybierz folder wyjściowy", "Choose output folder", "Ausgabeordner wählen", "Выбрать папку вывода"));

    set_accessible_name(controls.minutes_edit, tr("Długość jednego fragmentu w minutach", "Segment length in minutes", "Segmentlänge in Minuten", "Длина фрагмента в минутах"));
    set_accessible_description(
        controls.minutes_edit,
        tr(
            "Wpisz liczbę minut od 1 do 720. Domyślnie ustawiono 30 minut.",
            "Enter a value from 1 to 720 minutes. The default is 30 minutes.",
            "Geben Sie 1 bis 720 Minuten ein. Standard sind 30 Minuten.",
            "Введите от 1 до 720 минут. По умолчанию выбрано 30 минут.",
        ),
    );

    set_accessible_name(controls.format_combo, tr("Format wyjściowy", "Output format", "Ausgabeformat", "Формат вывода"));
    set_accessible_description(
        controls.format_combo,
        tr(
            "Wybierz zachowanie formatu wejściowego albo MP3, M4A, WAV, FLAC lub OPUS.",
            "Keep the input format or convert to MP3, M4A, WAV, FLAC or OPUS.",
            "Eingabeformat beibehalten oder in MP3, M4A, WAV, FLAC oder OPUS umwandeln.",
            "Сохраните исходный формат или преобразуйте в MP3, M4A, WAV, FLAC либо OPUS.",
        ),
    );

    set_accessible_name(controls.resource_combo, tr("Zużycie zasobów komputera", "Computer resource usage", "Ressourcennutzung", "Использование ресурсов"));
    set_accessible_description(controls.resource_combo, tr("Wybierz niskie, normalne albo szybkie przetwarzanie.", "Choose low, normal or fast processing.", "Wählen Sie niedrige, normale oder schnelle Verarbeitung.", "Выберите низкую, обычную или быструю обработку."));

    set_accessible_name(controls.bitrate_combo, tr("Bitrate pliku wynikowego", "Output bitrate", "Ausgabe-Bitrate", "Битрейт результата"));
    set_accessible_description(
        controls.bitrate_combo,
        tr(
            "Wybierz bitrate od 64 do 320 kb/s dla MP3, M4A i OPUS.",
            "Choose 64 to 320 kb/s for MP3, M4A and OPUS.",
            "Wählen Sie 64 bis 320 kb/s für MP3, M4A und OPUS.",
            "Выберите от 64 до 320 кбит/с для MP3, M4A и OPUS.",
        ),
    );

    set_accessible_name(controls.sample_rate_combo, tr("Częstotliwość próbkowania pliku wynikowego", "Output sample rate", "Abtastrate der Ausgabedatei", "Частота дискретизации результата"));
    set_accessible_description(
        controls.sample_rate_combo,
        tr(
            "Wybierz bez zmian, 44 100 Hz albo 48 000 Hz. OPUS wymaga 48 000 Hz lub opcji bez zmian.",
            "Choose unchanged, 44,100 Hz or 48,000 Hz. OPUS requires 48,000 Hz or unchanged.",
            "Wählen Sie unverändert, 44.100 Hz oder 48.000 Hz. OPUS benötigt 48.000 Hz oder unverändert.",
            "Выберите без изменений, 44 100 Гц или 48 000 Гц. Для OPUS требуется 48 000 Гц либо без изменений.",
        ),
    );

    set_accessible_name(controls.smart_check, tr("Tryb audiobook: wykrywaj rozdziały", "Audiobook mode: detect chapters", "Hörbuchmodus: Kapitel erkennen", "Режим аудиокниги: распознавать главы"));
    set_accessible_description(
        controls.smart_check,
        tr(
            "Najpierw używa rozdziałów zapisanych w pliku, a następnie pauz i lokalnego modelu Whisper. Ustawiona liczba minut ogranicza długość fragmentu.",
            "Uses embedded chapters first, then pauses and the local Whisper model. The selected minutes limit segment length.",
            "Verwendet zuerst eingebettete Kapitel, danach Pausen und das lokale Whisper-Modell. Die Minutenzahl begrenzt die Segmentlänge.",
            "Сначала используются встроенные главы, затем паузы и локальная модель Whisper. Выбранное число минут ограничивает длину фрагмента.",
        ),
    );

    set_accessible_name(controls.start_button, tr("Uruchom cięcie", "Start splitting", "Teilung starten", "Начать разделение"));
    set_accessible_name(controls.stop_button, tr("Zatrzymaj teraz", "Stop now", "Jetzt stoppen", "Остановить"));
    set_accessible_name(controls.update_button, tr("Sprawdź aktualizacje", "Check for updates", "Nach Updates suchen", "Проверить обновления"));
    set_accessible_description(controls.update_button, tr("Sprawdza i instaluje nowe wydania z GitHub Releases.", "Checks and installs new versions from GitHub Releases.", "Prüft und installiert neue Versionen aus GitHub Releases.", "Проверяет и устанавливает новые версии из GitHub Releases."));
    set_accessible_name(controls.support_revolut_button, tr("Wesprzyj przez Revolut", "Support via Revolut", "Über Revolut unterstützen", "Поддержать через Revolut"));
    set_accessible_description(controls.support_revolut_button, tr("Otwiera stronę wsparcia Revolut w przeglądarce.", "Opens the Revolut support page in your browser.", "Öffnet die Revolut-Unterstützungsseite im Browser.", "Открывает страницу поддержки Revolut в браузере."));
    set_accessible_name(controls.support_paypal_button, tr("Wesprzyj przez PayPal", "Support via PayPal", "Über PayPal unterstützen", "Поддержать через PayPal"));
    set_accessible_description(controls.support_paypal_button, tr("Otwiera stronę wsparcia PayPal w przeglądarce.", "Opens the PayPal support page in your browser.", "Öffnet die PayPal-Unterstützungsseite im Browser.", "Открывает страницу поддержки PayPal в браузере."));
    set_accessible_name(controls.language_combo, tr("Język interfejsu", "Interface language", "Sprache der Oberfläche", "Язык интерфейса"));
    set_accessible_description(controls.language_combo, tr("Zmienia język programu i zapisuje wybór w Majher.ini.", "Changes the program language and saves it in Majher.ini.", "Ändert die Programmsprache und speichert sie in Majher.ini.", "Меняет язык программы и сохраняет выбор в Majher.ini."));
    set_accessible_name(controls.theme_combo, tr("Motyw programu", "Program theme", "Programmdesign", "Тема программы"));
    set_accessible_description(controls.theme_combo, tr("Wybierz motyw systemowy, jasny lub ciemny. Ustawienie jest zapisywane w Majher.ini.", "Choose the system, light or dark theme. The setting is saved in Majher.ini.", "Wählen Sie das Systemdesign, ein helles oder dunkles Design. Die Einstellung wird in Majher.ini gespeichert.", "Выберите системную, светлую или тёмную тему. Настройка сохраняется в Majher.ini."));
    let version_text = format_tr("Wersja {}", "Version {}", "Version {}", "Версия {}", &[APP_VERSION.to_string()]);
    set_accessible_name(controls.version_label, &version_text);

    set_accessible_name(controls.progress_bar, tr("Postęp przetwarzania", "Processing progress", "Verarbeitungsfortschritt", "Ход обработки"));
    set_accessible_description(controls.progress_bar, tr("Postęp całego zadania od zera do stu procent.", "Progress of the whole job from zero to one hundred percent.", "Fortschritt des gesamten Auftrags von null bis hundert Prozent.", "Ход всего задания от нуля до ста процентов."));
    set_accessible_name(controls.log_edit, tr("Status i dziennik pracy", "Status and activity log", "Status und Arbeitsprotokoll", "Состояние и журнал работы"));
    set_accessible_description(controls.log_edit, tr("Pole z komunikatami programu dostępne klawiszem Tab.", "Program messages, reachable with the Tab key.", "Programmmeldungen, erreichbar mit der Tabulatortaste.", "Сообщения программы, доступные клавишей Tab."));
}

unsafe fn set_accessible_name(hwnd: Hwnd, name: &str) {
    set_accessible_property(hwnd, PROPID_ACC_NAME, name);
}

unsafe fn set_accessible_description(hwnd: Hwnd, description: &str) {
    set_accessible_property(hwnd, PROPID_ACC_DESCRIPTION, description);
}

unsafe fn set_accessible_property(hwnd: Hwnd, property: Guid, text: &str) {
    if hwnd == 0 {
        return;
    }

    let mut service: *mut IAccPropServices = null_mut();
    let hr = CoCreateInstance(
        &CLSID_ACC_PROP_SERVICES,
        null_mut(),
        CLSCTX_INPROC_SERVER,
        &IID_IACC_PROP_SERVICES,
        &mut service as *mut _ as *mut *mut core::ffi::c_void,
    );
    if hr < 0 || service.is_null() {
        return;
    }

    let value = wide(text);
    ((*(*service).lp_vtbl).set_hwnd_prop_str)(
        service,
        hwnd,
        OBJID_CLIENT,
        CHILDID_SELF,
        property,
        value.as_ptr(),
    );
    ((*(*service).lp_vtbl).release)(service);
}

unsafe fn add_combo_item(hwnd: Hwnd, text: &str) {
    let text = wide(text);
    SendMessageW(hwnd, CB_ADDSTRING, 0, text.as_ptr() as Lparam);
}

unsafe fn populate_theme_combo(hwnd: Hwnd) {
    SendMessageW(hwnd, CB_RESETCONTENT, 0, 0);
    for name in [
        tr("Zgodny z systemem", "Follow system", "Systemeinstellung", "Как в системе"),
        tr("Jasny", "Light", "Hell", "Светлая"),
        tr("Ciemny", "Dark", "Dunkel", "Тёмная"),
    ] {
        add_combo_item(hwnd, name);
    }
    SendMessageW(hwnd, CB_SETCURSEL, theme_preference() as Wparam, 0);
}

unsafe fn refresh_localized_combo(hwnd: Hwnd, items: &[&str]) {
    let selected = SendMessageW(hwnd, CB_GETCURSEL, 0, 0);
    SendMessageW(hwnd, CB_RESETCONTENT, 0, 0);
    for item in items {
        add_combo_item(hwnd, item);
    }
    if selected != CB_ERR {
        SendMessageW(hwnd, CB_SETCURSEL, selected as Wparam, 0);
    }
}

unsafe fn refresh_localized_ui(hwnd: Hwnd, controls: Controls) {
    set_window_text(controls.label_files, tr("Pliki audio do pocięcia", "Audio files to split", "Audiodateien teilen", "Аудиофайлы для разделения"));
    set_window_text(controls.add_button, tr("Dodaj pliki", "Add files", "Dateien hinzufügen", "Добавить файлы"));
    set_window_text(controls.remove_button, tr("Usuń zaznaczony", "Remove selected", "Auswahl entfernen", "Удалить выбранное"));
    set_window_text(controls.clear_button, tr("Wyczyść listę", "Clear list", "Liste leeren", "Очистить список"));
    set_window_text(controls.label_output, tr("Folder wyjściowy, puste pole oznacza folder obok pliku wejściowego", "Output folder; leave empty to save next to the input file", "Ausgabeordner; leer bedeutet neben der Eingabedatei", "Папка вывода; оставьте пустой для сохранения рядом с исходным файлом"));
    set_window_text(controls.output_button, tr("Wybierz folder", "Choose folder", "Ordner wählen", "Выбрать папку"));
    set_window_text(controls.label_minutes, tr("Długość jednego fragmentu w minutach", "Segment length in minutes", "Segmentlänge in Minuten", "Длина фрагмента в минутах"));
    set_window_text(controls.label_format, tr("Format wyjściowy", "Output format", "Ausgabeformat", "Формат вывода"));
    set_window_text(controls.label_resource, tr("Zużycie zasobów komputera", "Computer resource usage", "Ressourcennutzung", "Использование ресурсов"));
    set_window_text(controls.label_bitrate, tr("Bitrate pliku wynikowego", "Output bitrate", "Ausgabe-Bitrate", "Битрейт результата"));
    set_window_text(controls.label_sample_rate, tr("Częstotliwość próbkowania", "Sample rate", "Abtastrate", "Частота дискретизации"));
    set_window_text(controls.smart_check, tr("Tryb audiobook: wykrywaj rozdziały, części, kasety, strony i ścieżki", "Audiobook mode: detect chapters, parts, tapes, sides and tracks", "Hörbuchmodus: Kapitel, Teile, Kassetten, Seiten und Spuren erkennen", "Режим аудиокниги: распознавать главы, части, кассеты, стороны и дорожки"));
    set_window_text(controls.start_button, tr("Uruchom cięcie", "Start splitting", "Teilung starten", "Начать разделение"));
    set_window_text(controls.stop_button, tr("Zatrzymaj teraz", "Stop now", "Jetzt stoppen", "Остановить"));
    set_window_text(controls.update_button, tr("Sprawdź aktualizacje", "Check for updates", "Nach Updates suchen", "Проверить обновления"));
    set_window_text(controls.support_revolut_button, tr("Wsparcie: Revolut", "Support: Revolut", "Unterstützen: Revolut", "Поддержать: Revolut"));
    set_window_text(controls.support_paypal_button, tr("Wsparcie: PayPal", "Support: PayPal", "Unterstützen: PayPal", "Поддержать: PayPal"));
    set_window_text(controls.label_language, tr("Język interfejsu", "Interface language", "Sprache der Oberfläche", "Язык интерфейса"));
    set_window_text(controls.label_theme, tr("Motyw", "Theme", "Design", "Тема"));
    set_window_text(controls.version_label, &format_tr("Wersja {}", "Version {}", "Version {}", "Версия {}", &[APP_VERSION.to_string()]));
    set_window_text(controls.label_progress, tr("Postęp przetwarzania", "Processing progress", "Verarbeitungsfortschritt", "Ход обработки"));
    set_window_text(controls.label_log, tr("Status i dziennik pracy", "Status and activity log", "Status und Arbeitsprotokoll", "Состояние и журнал работы"));

    refresh_localized_combo(controls.format_combo, &[
        tr("Jak plik wejściowy, bez rekompresji", "Same as input, no recompression", "Wie Eingabe, ohne Neukodierung", "Как исходный, без перекодирования"),
        "MP3", "M4A AAC", "WAV", "FLAC", "OPUS",
    ]);
    refresh_localized_combo(controls.resource_combo, &[
        tr("Niskie, wolniej i lżej", "Low, slower and lighter", "Niedrig, langsamer und sparsamer", "Низкое, медленнее и легче"),
        tr("Normalne", "Normal", "Normal", "Обычное"),
        tr("Szybkie, więcej CPU", "Fast, more CPU", "Schnell, mehr CPU", "Быстрое, больше CPU"),
    ]);
    refresh_localized_combo(controls.sample_rate_combo, &[
        tr("Bez zmian, jak plik źródłowy", "Unchanged, same as source", "Unverändert, wie Quelldatei", "Без изменений, как в исходном файле"),
        "44 100 Hz", "48 000 Hz",
    ]);
    populate_theme_combo(controls.theme_combo);
    apply_accessible_names(hwnd, controls);
    apply_system_theme(hwnd, controls);
}

unsafe fn handle_language_change(hwnd: Hwnd) {
    let Some(state) = STATE.get() else { return };
    let controls = state.lock().unwrap().controls;
    let selected = SendMessageW(controls.language_combo, CB_GETCURSEL, 0, 0);
    let Some(next) = (selected != CB_ERR).then(|| language_from_index(selected as u8)).flatten() else { return };
    if next == language() {
        return;
    }
    if let Err(error) = save_settings(next, theme_preference()) {
        SendMessageW(controls.language_combo, CB_SETCURSEL, language() as Wparam, 0);
        show_message(hwnd, &format_tr(
            "Nie udało się zapisać ustawień.\n\n{}",
            "Could not save settings.\n\n{}",
            "Einstellungen konnten nicht gespeichert werden.\n\n{}",
            "Не удалось сохранить настройки.\n\n{}",
            &[error],
        ));
        return;
    }
    LANGUAGE.store(next as u8, Ordering::SeqCst);
    refresh_localized_ui(hwnd, controls);
    SetFocus(controls.language_combo);
}

unsafe fn handle_theme_change(hwnd: Hwnd) {
    let Some(state) = STATE.get() else { return };
    let controls = state.lock().unwrap().controls;
    let selected = SendMessageW(controls.theme_combo, CB_GETCURSEL, 0, 0);
    let Some(next) = (selected != CB_ERR).then(|| theme_from_index(selected as u8)).flatten() else { return };
    if next == theme_preference() {
        return;
    }
    if let Err(error) = save_settings(language(), next) {
        SendMessageW(controls.theme_combo, CB_SETCURSEL, theme_preference() as Wparam, 0);
        show_message(hwnd, &format_tr(
            "Nie udało się zapisać ustawień.\n\n{}",
            "Could not save settings.\n\n{}",
            "Einstellungen konnten nicht gespeichert werden.\n\n{}",
            "Не удалось сохранить настройки.\n\n{}",
            &[error],
        ));
        return;
    }
    THEME_PREFERENCE.store(next as u8, Ordering::SeqCst);
    apply_system_theme(hwnd, controls);
    SetFocus(controls.theme_combo);
}

unsafe fn handle_add_files(hwnd: Hwnd) {
    if let Some(paths) = choose_audio_files(hwnd) {
        if let Some(state) = STATE.get() {
            let mut state = state.lock().unwrap();
            for path in paths {
                if !state.files.iter().any(|existing| same_path(existing, &path)) {
                    state.files.push(path);
                }
            }
            refresh_file_list(state.controls.file_list, &state.files);
        }
    }
}

unsafe fn handle_remove_file() {
    if let Some(state) = STATE.get() {
        let mut state = state.lock().unwrap();
        let index = SendMessageW(state.controls.file_list, LB_GETCURSEL, 0, 0);
        if index != LB_ERR {
            let index = index as usize;
            if index < state.files.len() {
                state.files.remove(index);
                SendMessageW(state.controls.file_list, LB_DELETESTRING, index, 0);
            }
        }
    }
}

unsafe fn handle_clear_files() {
    if let Some(state) = STATE.get() {
        let mut state = state.lock().unwrap();
        state.files.clear();
        SendMessageW(state.controls.file_list, LB_RESETCONTENT, 0, 0);
    }
}

unsafe fn handle_browse_output(hwnd: Hwnd) {
    if let Some(path) = choose_folder(hwnd) {
        if let Some(state) = STATE.get() {
            let controls = state.lock().unwrap().controls;
            set_window_text(controls.output_edit, &path.to_string_lossy());
        }
    }
}

unsafe fn handle_start(hwnd: Hwnd) {
    let request = {
        let Some(state_lock) = STATE.get() else {
            return;
        };
        let mut state = state_lock.lock().unwrap();
        if state.running {
            return;
        }
        if state.updating {
            show_message(hwnd, tr("Poczekaj na zakończenie sprawdzania aktualizacji.", "Wait for the update check to finish.", "Warten Sie, bis die Updateprüfung abgeschlossen ist.", "Дождитесь завершения проверки обновлений."));
            return;
        }
        if state.files.is_empty() {
            show_message(hwnd, tr("Dodaj przynajmniej jeden plik audio.", "Add at least one audio file.", "Fügen Sie mindestens eine Audiodatei hinzu.", "Добавьте хотя бы один аудиофайл."));
            return;
        }

        let minutes_text = get_window_text(state.controls.minutes_edit);
        let minutes = minutes_text.trim().replace(',', ".").parse::<f64>().unwrap_or(0.0);
        if !(1.0..=720.0).contains(&minutes) {
            show_message(hwnd, tr("Podaj długość fragmentu od 1 do 720 minut.", "Enter a segment length from 1 to 720 minutes.", "Geben Sie eine Segmentlänge von 1 bis 720 Minuten ein.", "Введите длину фрагмента от 1 до 720 минут."));
            return;
        }

        let output_text = get_window_text(state.controls.output_edit);
        let output_root = if output_text.trim().is_empty() {
            None
        } else {
            Some(PathBuf::from(output_text.trim()))
        };

        let format = match SendMessageW(state.controls.format_combo, CB_GETCURSEL, 0, 0) {
            1 => OutputFormat::Mp3,
            2 => OutputFormat::M4a,
            3 => OutputFormat::Wav,
            4 => OutputFormat::Flac,
            5 => OutputFormat::Opus,
            CB_ERR => OutputFormat::Same,
            _ => OutputFormat::Same,
        };

        let resource = match SendMessageW(state.controls.resource_combo, CB_GETCURSEL, 0, 0) {
            0 => ResourceProfile::Low,
            2 => ResourceProfile::Fast,
            _ => ResourceProfile::Normal,
        };

        let bitrate_kbps = match SendMessageW(state.controls.bitrate_combo, CB_GETCURSEL, 0, 0) {
            0 => 64,
            1 => 96,
            3 => 160,
            4 => 192,
            5 => 256,
            6 => 320,
            _ => 128,
        };
        let sample_rate = match SendMessageW(state.controls.sample_rate_combo, CB_GETCURSEL, 0, 0) {
            1 => Some(44_100),
            2 => Some(48_000),
            _ => None,
        };
        if matches!(format, OutputFormat::Opus) && sample_rate == Some(44_100) {
            show_message(
                hwnd,
                tr("Format OPUS nie obsługuje częstotliwości 44 100 Hz. Wybierz 48 000 Hz albo Bez zmian.", "OPUS does not support 44,100 Hz. Choose 48,000 Hz or Unchanged.", "OPUS unterstützt 44.100 Hz nicht. Wählen Sie 48.000 Hz oder Unverändert.", "OPUS не поддерживает 44 100 Гц. Выберите 48 000 Гц или Без изменений."),
            );
            SetFocus(state.controls.sample_rate_combo);
            return;
        }

        let smart = SendMessageW(state.controls.smart_check, BM_GETCHECK, 0, 0) == BST_CHECKED;
        let nvidia_detected = smart && nvidia_available();
        let mut prefer_nvidia = nvidia_detected;
        if smart && !chapter_detector_components_present(prefer_nvidia) {
            if nvidia_detected {
                prefer_nvidia = match ask_detector_hardware(hwnd) {
                    Some(value) => value,
                    None => return,
                };
            } else {
                if !ask_yes_no(
                    hwnd,
                    tr(
                        "Pierwsze użycie wykrywania rozdziałów wymaga pobrania około 190 MB: modelu i silnika CPU. Pliki zostaną zapisane w katalogu tools\\chapter-detector obok programu.\n\nNie wykryto zgodnej karty NVIDIA, ale rozpoznawanie będzie w pełni działało na procesorze.\n\nCzy rozpocząć pobieranie i cięcie?",
                        "The first use of chapter detection requires downloading about 190 MB: the model and CPU engine. Files will be saved in tools\\chapter-detector next to the program.\n\nNo compatible NVIDIA card was detected, but recognition will work fully on the CPU.\n\nStart downloading and splitting?",
                        "Für die erste Kapitelerkennung müssen etwa 190 MB heruntergeladen werden: Modell und CPU-Engine. Die Dateien werden in tools\\chapter-detector neben dem Programm gespeichert.\n\nKeine kompatible NVIDIA-Karte wurde erkannt, die Erkennung funktioniert jedoch vollständig auf der CPU.\n\nDownload und Teilung starten?",
                        "Для первого распознавания глав требуется загрузить около 190 МБ: модель и движок CPU. Файлы будут сохранены в tools\\chapter-detector рядом с программой.\n\nСовместимая видеокарта NVIDIA не найдена, но распознавание будет полностью работать на CPU.\n\nНачать загрузку и разделение?",
                    ),
                ) {
                    return;
                }
            }
        }
        let cancel = Arc::new(AtomicBool::new(false));

        set_window_text(state.controls.log_edit, "");
        set_progress(state.controls.progress_bar, 0);
        state.running = true;
        state.cancel = Some(cancel.clone());
        set_controls_running(state.controls, true);
        SetFocus(state.controls.stop_button);

        SplitRequest {
            files: state.files.clone(),
            output_root,
            segment_seconds: minutes * 60.0,
            format,
            resource,
            bitrate_kbps,
            sample_rate,
            smart,
            prefer_nvidia,
            ffmpeg: PathBuf::new(),
            ffprobe: PathBuf::new(),
            cancel,
        }
    };

    thread::spawn(move || {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            run_worker(hwnd, request)
        }));
        let summary = match result {
            Ok(summary) => summary,
            Err(payload) => {
                let details = if let Some(text) = payload.downcast_ref::<&str>() {
                    (*text).to_string()
                } else if let Some(text) = payload.downcast_ref::<String>() {
                    text.clone()
                } else {
                    "nieznany błąd wewnętrzny".to_string()
                };
                post_log(hwnd, &format!("Błąd wewnętrzny: {}. Program bezpiecznie zakończył zadanie.", details));
                CompletionSummary {
                    errors: 1,
                    ..CompletionSummary::default()
                }
            }
        };
        unsafe {
            PostMessageW(
                hwnd,
                WM_APP_DONE,
                0,
                Box::into_raw(Box::new(summary)) as Lparam,
            );
        }
    });
}

unsafe fn handle_stop() {
    if let Some(state) = STATE.get() {
        let state = state.lock().unwrap();
        if let Some(cancel) = &state.cancel {
            cancel.store(true, Ordering::SeqCst);
            append_log(
                state.controls.log_edit,
                "Zatrzymanie zostało zlecone. Kończę bieżący proces FFmpeg lub modelu...",
            );
        }
    }
}

unsafe fn handle_check_update(hwnd: Hwnd, manual: bool) {
    let Some(state_lock) = STATE.get() else {
        return;
    };
    {
        let mut state = state_lock.lock().unwrap();
        if state.running {
            if manual {
                show_message(hwnd, tr("Aktualizację można sprawdzić po zakończeniu cięcia.", "Updates can be checked after splitting is finished.", "Nach Abschluss der Teilung kann nach Updates gesucht werden.", "Проверить обновления можно после завершения разделения."));
            }
            return;
        }
        if state.updating {
            return;
        }
        state.updating = true;
        EnableWindow(state.controls.update_button, 0);
        append_log(
            state.controls.log_edit,
            &format!("Sprawdzam aktualizację. Obecna wersja: {}.", APP_VERSION),
        );
    }

    thread::spawn(move || {
        let outcome = match check_and_download_update() {
            Ok(Some((version, changelog, downloaded_file))) => UpdateOutcome::Ready {
                version,
                changelog,
                downloaded_file,
            },
            Ok(None) => UpdateOutcome::Current,
            Err(error) => UpdateOutcome::Error(error),
        };
        let result = UpdateResult { outcome, manual };
        unsafe {
            PostMessageW(
                hwnd,
                WM_APP_UPDATE_RESULT,
                0,
                Box::into_raw(Box::new(result)) as Lparam,
            );
        }
    });
}

fn run_worker(hwnd: Hwnd, mut request: SplitRequest) -> CompletionSummary {
    let mut summary = CompletionSummary::default();
    post_log(hwnd, &format!("Start programu {}.", APP_TITLE));
    post_log(hwnd, &format!("Liczba plików: {}. Długość fragmentu: {:.0} minut.", request.files.len(), request.segment_seconds / 60.0));
    match ensure_ffmpeg(hwnd) {
        Ok((ffmpeg, ffprobe)) => {
            request.ffmpeg = ffmpeg;
            request.ffprobe = ffprobe;
        }
        Err(error) => {
            post_log(hwnd, &format!("Błąd przygotowania FFmpeg: {}", error));
            summary.errors = 1;
            return summary;
        }
    }
    post_log(hwnd, &format!("FFmpeg: {}", request.ffmpeg.to_string_lossy()));
    post_log(hwnd, &format!("FFprobe: {}", request.ffprobe.to_string_lossy()));
    post_log(hwnd, &format!("Bitrate: {} kb/s.", request.bitrate_kbps));
    post_log(
        hwnd,
        &format!(
            "Częstotliwość: {}.",
            request
                .sample_rate
                .map(|value| format!("{} Hz", value))
                .unwrap_or_else(|| "bez zmian".to_string())
        ),
    );

    let mut chapter_detector = if request.smart {
        match ensure_chapter_detector(hwnd, request.prefer_nvidia, &request.cancel) {
            Ok(detector) => Some(detector),
            Err(error) if request.cancel.load(Ordering::SeqCst) => {
                post_log(hwnd, &format!("Zatrzymano przygotowanie wykrywania rozdziałów: {}.", error));
                summary.cancelled = true;
                return summary;
            }
            Err(error) => {
                post_log(
                    hwnd,
                    &format!(
                        "Nie udało się przygotować modelu rozpoznawania. Pozostaje wykrywanie pauz: {}",
                        error
                    ),
                );
                None
            }
        }
    } else {
        None
    };

    let file_count = request.files.len().max(1);
    for (file_index, file) in request.files.iter().enumerate() {
        if request.cancel.load(Ordering::SeqCst) {
            post_log(hwnd, "Przerwano przed kolejnym plikiem.");
            break;
        }

        let progress_start = ((file_index * 1000) / file_count) as u32;
        let progress_end = (((file_index + 1) * 1000) / file_count) as u32;
        post_progress(hwnd, progress_start);

        post_log(hwnd, "");
        post_log(hwnd, &format!("Plik: {}", file.to_string_lossy()));
        match process_file(
            hwnd,
            file,
            &request,
            chapter_detector.as_mut(),
            progress_start,
            progress_end,
            &mut summary.output_files_created,
        ) {
            Ok(output_directory) if !request.cancel.load(Ordering::SeqCst) => {
                summary.input_files_finished += 1;
                if !summary
                    .output_directories
                    .iter()
                    .any(|existing| same_path(existing, &output_directory))
                {
                    summary.output_directories.push(output_directory);
                }
                post_progress(hwnd, progress_end);
                post_log(hwnd, "Plik zakończony.");
            }
            Ok(_) => post_log(hwnd, "Plik zatrzymany przed końcem."),
            Err(err) if request.cancel.load(Ordering::SeqCst) => {
                post_log(hwnd, &format!("Zatrzymano: {}", err));
            }
            Err(err) => {
                summary.errors += 1;
                post_log(hwnd, &format!("Błąd: {}", err));
            }
        }
    }

    summary.cancelled = request.cancel.load(Ordering::SeqCst);
    if summary.cancelled {
        post_log(hwnd, "Zatrzymano zadanie.");
    } else {
        post_log(hwnd, "Gotowe.");
    }
    summary
}

fn process_file(
    hwnd: Hwnd,
    file: &Path,
    request: &SplitRequest,
    mut chapter_detector: Option<&mut ChapterDetector>,
    progress_start: u32,
    progress_end: u32,
    output_files_created: &mut usize,
) -> Result<PathBuf, String> {
    post_progress(hwnd, progress_start);
    if !file.exists() {
        return Err("plik nie istnieje".to_string());
    }

    let mut duration = probe_duration(&request.ffprobe, file, request.resource)?;
    if duration <= 1.0 {
        return Err("nie udało się odczytać poprawnej długości pliku".to_string());
    }
    post_log(hwnd, &format!("Długość pliku: {}.", human_duration(duration)));
    post_progress(hwnd, progress_between(progress_start, progress_end, 20));

    let output_dir = output_directory(file, request.output_root.as_deref())?;
    fs::create_dir_all(&output_dir).map_err(|err| format!("nie można utworzyć folderu wyjściowego: {}", err))?;
    post_log(hwnd, &format!("Folder wyjściowy: {}", output_dir.to_string_lossy()));

    let embedded_chapters = if request.smart {
        match probe_chapter_starts(&request.ffprobe, file, request.resource) {
            Ok(points) if !points.is_empty() => {
                post_log(
                    hwnd,
                    &format!(
                        "Znaleziono rozdziały zapisane w pliku: {}. Używam ich jako najpewniejszych granic.",
                        points.len() + 1
                    ),
                );
                points
            }
            Ok(_) => {
                post_log(hwnd, "Plik nie zawiera zapisanej listy rozdziałów.");
                Vec::new()
            }
            Err(error) => {
                post_log(
                    hwnd,
                    &format!("Nie udało się odczytać zapisanych rozdziałów: {}", error),
                );
                Vec::new()
            }
        }
    } else {
        Vec::new()
    };

    let silence_ranges = if request.smart {
        post_log(hwnd, "Tryb audiobook: szukam dłuższych pauz, również w nagraniach z szumem taśmy.");
        match detect_silence_points(
            &request.ffmpeg,
            file,
            duration,
            request.resource,
            &request.cancel,
            hwnd,
            progress_between(progress_start, progress_end, 20),
            progress_between(progress_start, progress_end, 260),
        ) {
            Ok((points, scanned_duration)) => {
                if scanned_duration > duration + 1.0 {
                    post_log(
                        hwnd,
                        &format!(
                            "Skorygowano długość według pełnego skanowania FFmpeg: {} zamiast {}.",
                            human_duration(scanned_duration),
                            human_duration(duration)
                        ),
                    );
                    duration = scanned_duration;
                }
                post_log(hwnd, &format!("Znalezione dłuższe pauzy: {}.", points.len()));
                points
            }
            Err(err) if request.cancel.load(Ordering::SeqCst) => return Err(err),
            Err(err) => {
                post_log(hwnd, &format!("Nie udało się wykryć ciszy, używam równego cięcia. Szczegóły: {}", err));
                Vec::new()
            }
        }
    } else {
        post_progress(hwnd, progress_between(progress_start, progress_end, 500));
        Vec::new()
    };

    let silence_points: Vec<f64> = silence_ranges
        .iter()
        .copied()
        .map(SilencePoint::midpoint)
        .collect();
    let recognized_chapters = if request.smart && embedded_chapters.is_empty() {
        if let Some(detector) = chapter_detector.as_deref_mut() {
            match recognize_audiobook_boundaries(
                detector,
                &request.ffmpeg,
                file,
                duration,
                &silence_ranges,
                request.resource,
                &request.cancel,
                hwnd,
                progress_between(progress_start, progress_end, 260),
                progress_between(progress_start, progress_end, 500),
            ) {
                Ok(points) => points,
                Err(error) if request.cancel.load(Ordering::SeqCst) => return Err(error),
                Err(error) => {
                    post_log(
                        hwnd,
                        &format!(
                            "Rozpoznawanie zapowiedzi nie powiodło się. Używam samych pauz: {}",
                            error
                        ),
                    );
                    Vec::new()
                }
            }
        } else {
            post_progress(hwnd, progress_between(progress_start, progress_end, 500));
            Vec::new()
        }
    } else {
        post_progress(hwnd, progress_between(progress_start, progress_end, 500));
        Vec::new()
    };

    let chapter_boundaries = if embedded_chapters.is_empty() {
        recognized_chapters
    } else {
        embedded_chapters
    };
    let segments = if request.smart && !chapter_boundaries.is_empty() {
        post_log(
            hwnd,
            &format!(
                "Pewne granice audiobooka użyte do cięcia: {}.",
                chapter_boundaries.len()
            ),
        );
        build_audiobook_segments(
            duration,
            request.segment_seconds,
            &chapter_boundaries,
            &silence_points,
        )
    } else {
        if request.smart {
            post_log(
                hwnd,
                "Nie rozpoznano jednoznacznej zapowiedzi rozdziału. Stosuję bezpieczny podział według długości i pobliskich pauz.",
            );
        }
        build_segments(duration, request.segment_seconds, &silence_points)
    };
    if segments.is_empty() {
        return Err("nie wyliczono żadnego fragmentu".to_string());
    }
    post_log(hwnd, &format!("Liczba fragmentów: {}.", segments.len()));

    for (index, (start, end)) in segments.iter().copied().enumerate() {
        if request.cancel.load(Ordering::SeqCst) {
            post_log(hwnd, "Zatrzymano przed kolejnym fragmentem.");
            break;
        }

        let output_path = segment_path(file, &output_dir, index + 1, request.format);
        let cut_progress_start = progress_between(
            progress_start,
            progress_end,
            500 + ((500 * index) / segments.len()) as u32,
        );
        let cut_progress_end = progress_between(
            progress_start,
            progress_end,
            500 + ((500 * (index + 1)) / segments.len()) as u32,
        );
        post_log(
            hwnd,
            &format!(
                "Fragment {}/{}: {} - {} -> {}",
                index + 1,
                segments.len(),
                human_duration(start),
                human_duration(end),
                output_path.file_name().unwrap_or_default().to_string_lossy()
            ),
        );

        match cut_segment(
            &request.ffmpeg,
            file,
            &output_path,
            start,
            end - start,
            request.format,
            request.resource,
            request.bitrate_kbps,
            request.sample_rate,
            &request.cancel,
            hwnd,
            cut_progress_start,
            cut_progress_end,
        ) {
            Ok(()) => {
                *output_files_created += 1;
            }
            Err(err) if request.cancel.load(Ordering::SeqCst) => return Err(err),
            Err(err) if matches!(request.format, OutputFormat::Same) => {
                let fallback = output_path.with_extension("mp3");
                post_log(hwnd, &format!("Cięcie bez rekompresji nie powiodło się, próbuję MP3. Szczegóły: {}", err));
                cut_segment(
                    &request.ffmpeg,
                    file,
                    &fallback,
                    start,
                    end - start,
                    OutputFormat::Mp3,
                    request.resource,
                    request.bitrate_kbps,
                    request.sample_rate,
                    &request.cancel,
                    hwnd,
                    cut_progress_start,
                    cut_progress_end,
                )?;
                *output_files_created += 1;
            }
            Err(err) => return Err(err),
        }
    }

    Ok(output_dir)
}

fn probe_duration(ffprobe: &Path, file: &Path, resource: ResourceProfile) -> Result<f64, String> {
    let mut command = Command::new(ffprobe);
    command
        .arg("-v")
        .arg("error")
        .arg("-show_entries")
        .arg("format=duration:stream=duration")
        .arg("-of")
        .arg("default=nokey=1:noprint_wrappers=1")
        .arg(file);
    let output = run_capture(command, resource)?;
    if !output.status.success() {
        return Err(short_process_error("ffprobe", &output));
    }
    let text = String::from_utf8_lossy(&output.stdout);
    parse_longest_duration(&text)
        .ok_or_else(|| format!("ffprobe zwrócił nieczytelną długość: {}", text.trim()))
}

fn parse_longest_duration(text: &str) -> Option<f64> {
    text.lines()
        .filter_map(|line| line.trim().parse::<f64>().ok())
        .filter(|value| value.is_finite() && *value > 0.0)
        .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
}

fn probe_chapter_starts(
    ffprobe: &Path,
    file: &Path,
    resource: ResourceProfile,
) -> Result<Vec<f64>, String> {
    let mut command = Command::new(ffprobe);
    command
        .arg("-v")
        .arg("error")
        .arg("-show_chapters")
        .arg("-show_entries")
        .arg("chapter=start_time")
        .arg("-of")
        .arg("compact=p=0:nk=0")
        .arg(file);
    let output = run_capture(command, resource)?;
    if !output.status.success() {
        return Err(short_process_error("ffprobe", &output));
    }
    Ok(parse_chapter_starts(&String::from_utf8_lossy(&output.stdout)))
}

fn parse_chapter_starts(text: &str) -> Vec<f64> {
    let mut result: Vec<f64> = text
        .lines()
        .flat_map(|line| line.split('|'))
        .filter_map(|field| field.trim().strip_prefix("start_time="))
        .filter_map(|value| value.parse::<f64>().ok())
        .filter(|value| value.is_finite() && *value > 1.0)
        .collect();
    result.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    result.dedup_by(|a, b| (*a - *b).abs() < 0.5);
    result
}

fn recognize_audiobook_boundaries(
    detector: &mut ChapterDetector,
    ffmpeg: &Path,
    file: &Path,
    duration: f64,
    silence_ranges: &[SilencePoint],
    resource: ResourceProfile,
    cancel: &AtomicBool,
    hwnd: Hwnd,
    progress_start: u32,
    progress_end: u32,
) -> Result<Vec<f64>, String> {
    let candidates = recognition_candidates(silence_ranges, duration);
    if candidates.is_empty() {
        post_log(
            hwnd,
            "Brak wystarczająco długich pauz do sprawdzenia zapowiedzi modelem.",
        );
        post_progress(hwnd, progress_end);
        return Ok(Vec::new());
    }
    post_log(
        hwnd,
        &format!(
            "Sprawdzam modelem {} miejsc mogących rozpoczynać nowy rozdział.",
            candidates.len()
        ),
    );

    let temporary = std::env::temp_dir().join(format!(
        "majher-chapter-scan-{}-{}",
        std::process::id(),
        cache_buster()
    ));
    fs::create_dir_all(&temporary)
        .map_err(|error| format!("nie można utworzyć katalogu roboczego modelu: {}", error))?;

    let mut prepared = Vec::new();
    for (index, candidate) in candidates.iter().copied().enumerate() {
        if cancel.load(Ordering::SeqCst) {
            let _ = fs::remove_dir_all(&temporary);
            return Err("zatrzymano rozpoznawanie rozdziałów".to_string());
        }
        let local_start = ((index * 350) / candidates.len()) as u32;
        post_progress(
            hwnd,
            progress_between(progress_start, progress_end, local_start),
        );

        let window_start = (candidate.midpoint() - 12.0).max(0.0);
        let window_duration = (45.0f64).min(duration - window_start).max(1.0);
        let wav = temporary.join(format!("okno-{:04}.wav", index));
        let prefix = temporary.join(format!("tekst-{:04}", index));
        match extract_recognition_window(
            ffmpeg,
            file,
            &wav,
            window_start,
            window_duration,
            resource,
            cancel,
            hwnd,
        ) {
            Ok(()) => {}
            Err(error) if cancel.load(Ordering::SeqCst) => {
                let _ = fs::remove_dir_all(&temporary);
                return Err(error);
            }
            Err(error) => {
                post_log(
                    hwnd,
                    &format!(
                        "Pomijam próbkę w czasie {}: {}",
                        human_duration(candidate.midpoint()),
                        error
                    ),
                );
                continue;
            }
        }
        prepared.push((candidate, wav, prefix));
    }

    if prepared.is_empty() {
        let _ = fs::remove_dir_all(&temporary);
        post_progress(hwnd, progress_end);
        post_log(hwnd, "Nie udało się przygotować żadnej próbki do rozpoznania.");
        return Ok(Vec::new());
    }

    post_log(
        hwnd,
        &format!(
            "Przygotowano {} próbek. Uruchamiam jeden wspólny wsad {}, aby model pozostał załadowany w pamięci.",
            prepared.len(),
            if detector.using_nvidia {
                "NVIDIA CUDA"
            } else {
                "CPU"
            }
        ),
    );
    post_progress(
        hwnd,
        progress_between(progress_start, progress_end, 350),
    );

    let jobs: Vec<(PathBuf, PathBuf)> = prepared
        .iter()
        .map(|(_, wav, prefix)| (wav.clone(), prefix.clone()))
        .collect();
    let transcripts = match transcribe_recognition_batch(
        detector,
        &jobs,
        resource,
        cancel,
        hwnd,
        &temporary,
    ) {
        Ok(results) => results,
        Err(error) if cancel.load(Ordering::SeqCst) => {
            let _ = fs::remove_dir_all(&temporary);
            return Err(error);
        }
        Err(error) => {
            post_log(
                hwnd,
                &format!("Model nie przeanalizował przygotowanych próbek: {}", error),
            );
            let _ = fs::remove_dir_all(&temporary);
            post_progress(hwnd, progress_end);
            return Ok(Vec::new());
        }
    };

    let mut boundaries = Vec::new();
    let mut last_recognized = -120.0f64;
    let prepared_len = prepared.len();
    for (index, ((candidate, wav, prefix), transcript)) in
        prepared.into_iter().zip(transcripts).enumerate()
    {
        if cancel.load(Ordering::SeqCst) {
            let _ = fs::remove_dir_all(&temporary);
            return Err("zatrzymano rozpoznawanie rozdziałów".to_string());
        }
        let local_progress = 350 + ((index * 650) / prepared_len) as u32;
        post_progress(
            hwnd,
            progress_between(progress_start, progress_end, local_progress),
        );
        let transcript = match transcript {
            Ok(text) => text,
            Err(error) => {
                post_log(
                    hwnd,
                    &format!(
                        "Model pominął próbkę w czasie {}: {}",
                        human_duration(candidate.midpoint()),
                        error
                    ),
                );
                continue;
            }
        };
        let score = audiobook_marker_score(&transcript);
        let accepted = score >= 2 || (score == 1 && candidate.duration() >= 2.5);
        if accepted && candidate.midpoint() - last_recognized >= 30.0 {
            let snippet: String = transcript
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
                .chars()
                .take(180)
                .collect();
            post_log(
                hwnd,
                &format!(
                    "Rozpoznano granicę {}: {}",
                    human_duration(candidate.midpoint()),
                    snippet
                ),
            );
            boundaries.push(candidate.midpoint());
            last_recognized = candidate.midpoint();
        }
        let _ = fs::remove_file(&wav);
        let _ = fs::remove_file(prefix.with_extension("txt"));
    }
    let _ = fs::remove_dir_all(&temporary);
    post_progress(hwnd, progress_end);
    post_log(
        hwnd,
        &format!("Rozpoznane zapowiedzi i granice audiobooka: {}.", boundaries.len()),
    );
    Ok(boundaries)
}

fn recognition_candidates(points: &[SilencePoint], duration: f64) -> Vec<SilencePoint> {
    let mut result: Vec<SilencePoint> = Vec::new();
    for point in points.iter().copied().filter(|point| {
        point.duration() >= 1.8
            && point.midpoint() >= 5.0
            && point.midpoint() <= duration - 5.0
    }) {
        if let Some(last) = result.last_mut() {
            if point.midpoint() - last.midpoint() < 45.0 {
                if point.duration() > last.duration() {
                    *last = point;
                }
                continue;
            }
        }
        result.push(point);
    }
    if result.len() > 500 {
        let mut strongest = result;
        strongest.sort_by(|a, b| {
            b.duration()
                .partial_cmp(&a.duration())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        strongest.truncate(500);
        strongest.sort_by(|a, b| {
            a.midpoint()
                .partial_cmp(&b.midpoint())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        strongest
    } else {
        result
    }
}

fn extract_recognition_window(
    ffmpeg: &Path,
    input: &Path,
    output: &Path,
    start: f64,
    duration: f64,
    resource: ResourceProfile,
    cancel: &AtomicBool,
    hwnd: Hwnd,
) -> Result<(), String> {
    let mut command = Command::new(ffmpeg);
    command
        .arg("-hide_banner")
        .arg("-nostats")
        .arg("-y")
        .arg("-nostdin")
        .arg("-ss")
        .arg(seconds_arg(start))
        .arg("-i")
        .arg(input)
        .arg("-t")
        .arg(seconds_arg(duration))
        .arg("-map")
        .arg("0:a:0")
        .arg("-vn")
        .arg("-ac")
        .arg("1")
        .arg("-ar")
        .arg("16000")
        .arg("-c:a")
        .arg("pcm_s16le")
        .arg(output);
    let report = run_ffmpeg(
        command,
        resource,
        cancel,
        hwnd,
        "Przygotowanie próbki rozdziału",
        None,
    )?;
    if report.status.success() {
        Ok(())
    } else {
        Err(short_stderr_error("ffmpeg", &report.stderr))
    }
}

#[cfg(test)]
fn transcribe_recognition_window(
    detector: &mut ChapterDetector,
    wav: &Path,
    output_prefix: &Path,
    resource: ResourceProfile,
    cancel: &AtomicBool,
    hwnd: Hwnd,
) -> Result<String, String> {
    let work_directory = output_prefix
        .parent()
        .ok_or_else(|| "próbka modelu nie ma katalogu roboczego".to_string())?;
    let jobs = vec![(wav.to_path_buf(), output_prefix.to_path_buf())];
    let mut results = transcribe_recognition_batch(
        detector,
        &jobs,
        resource,
        cancel,
        hwnd,
        work_directory,
    )?;
    results
        .pop()
        .unwrap_or_else(|| Err("model nie zwrócił wyniku próbki".to_string()))
}

fn transcribe_recognition_batch(
    detector: &mut ChapterDetector,
    jobs: &[(PathBuf, PathBuf)],
    resource: ResourceProfile,
    cancel: &AtomicBool,
    hwnd: Hwnd,
    work_directory: &Path,
) -> Result<Vec<Result<String, String>>, String> {
    let first = run_whisper_batch(
        &detector.primary_exe,
        &detector.model,
        jobs,
        resource,
        cancel,
        hwnd,
        work_directory,
        detector.using_nvidia,
    );
    if first.is_ok() || !detector.using_nvidia || cancel.load(Ordering::SeqCst) {
        return first;
    }

    let gpu_error = first.unwrap_err();
    post_log(
        hwnd,
        &format!(
            "Przyspieszenie NVIDIA nie zadziałało z tym sterownikiem lub kartą. Przechodzę na CPU: {}",
            gpu_error
        ),
    );
    detector.primary_exe = detector.cpu_exe.clone();
    detector.using_nvidia = false;
    for (_, output_prefix) in jobs {
        let _ = fs::remove_file(output_prefix.with_extension("txt"));
    }
    run_whisper_batch(
        &detector.cpu_exe,
        &detector.model,
        jobs,
        resource,
        cancel,
        hwnd,
        work_directory,
        false,
    )
}

fn whisper_batch_response(
    model: &Path,
    jobs: &[(PathBuf, PathBuf)],
    resource: ResourceProfile,
    use_flash_attention: bool,
) -> String {
    let mut arguments = vec![
        "-m".to_string(),
        model.to_string_lossy().into_owned(),
        "-l".to_string(),
        "pl".to_string(),
        "-t".to_string(),
        resource.thread_count().to_string(),
        "-bo".to_string(),
        "1".to_string(),
        "-bs".to_string(),
        "1".to_string(),
        "-otxt".to_string(),
        "-np".to_string(),
        "-nt".to_string(),
    ];
    if use_flash_attention {
        arguments.push("-fa".to_string());
    }
    for (wav, output_prefix) in jobs {
        arguments.push("-f".to_string());
        arguments.push(wav.to_string_lossy().into_owned());
        arguments.push("-of".to_string());
        arguments.push(output_prefix.to_string_lossy().into_owned());
    }
    arguments.join("\n")
}

fn run_whisper_batch(
    whisper: &Path,
    model: &Path,
    jobs: &[(PathBuf, PathBuf)],
    resource: ResourceProfile,
    cancel: &AtomicBool,
    hwnd: Hwnd,
    work_directory: &Path,
    use_flash_attention: bool,
) -> Result<Vec<Result<String, String>>, String> {
    if jobs.is_empty() {
        return Ok(Vec::new());
    }
    for (_, output_prefix) in jobs {
        let _ = fs::remove_file(output_prefix.with_extension("txt"));
    }

    let response_file = work_directory.join(format!(
        "whisper-wsad-{}-{}.rsp",
        std::process::id(),
        cache_buster()
    ));
    fs::write(
        &response_file,
        whisper_batch_response(model, jobs, resource, use_flash_attention),
    )
    .map_err(|error| format!("nie można przygotować wsadu modelu: {}", error))?;

    let mut command = Command::new(whisper);
    command.arg(format!("@{}", response_file.to_string_lossy()));
    let report = run_ffmpeg(
        command,
        resource,
        cancel,
        hwnd,
        "Rozpoznawanie zapowiedzi rozdziału",
        None,
    );
    let _ = fs::remove_file(&response_file);
    let report = report?;
    if !report.status.success() {
        return Err(short_stderr_error("whisper-cli", &report.stderr));
    }
    let results: Vec<Result<String, String>> = jobs
        .iter()
        .map(|(_, output_prefix)| {
            fs::read_to_string(output_prefix.with_extension("txt"))
                .map_err(|error| format!("model nie utworzył czytelnego wyniku: {}", error))
        })
        .collect();
    if results.iter().all(Result::is_err) {
        return Err("model zakończył pracę, ale nie utworzył żadnego wyniku".to_string());
    }
    Ok(results)
}

fn normalize_marker_text(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|ch| match ch {
            'ą' => 'a',
            'ć' => 'c',
            'ę' => 'e',
            'ł' => 'l',
            'ń' => 'n',
            'ó' => 'o',
            'ś' => 's',
            'ź' | 'ż' => 'z',
            ch if ch.is_alphanumeric() => ch,
            _ => ' ',
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn audiobook_marker_score(text: &str) -> u8 {
    let normalized = normalize_marker_text(text);
    let words: Vec<&str> = normalized.split_whitespace().collect();
    if words
        .iter()
        .any(|word| matches!(*word, "prolog" | "epilog" | "poslowie" | "przedmowa"))
    {
        return 2;
    }
    for (index, word) in words.iter().enumerate() {
        if matches!(
            *word,
            "rozdzial" | "czesc" | "ksiega" | "tom" | "kaseta" | "strona" | "sciezka"
        ) {
            let lower = index.saturating_sub(2);
            let upper = (index + 5).min(words.len());
            if words[lower..upper].iter().any(|value| is_audiobook_number(value))
                || (index > 0 && words[index - 1] == "koniec")
            {
                return 2;
            }
            if matches!(*word, "rozdzial" | "ksiega" | "kaseta") {
                return 1;
            }
        }
    }
    0
}

fn is_audiobook_number(value: &str) -> bool {
    value.chars().any(|ch| ch.is_ascii_digit())
        || matches!(
            value,
            "a"
                | "b"
                | "i"
                | "ii"
                | "iii"
                | "iv"
                | "v"
                | "vi"
                | "vii"
                | "viii"
                | "ix"
                | "x"
                | "pierwsza"
                | "pierwszy"
                | "pierwsze"
                | "druga"
                | "drugi"
                | "drugie"
                | "trzecia"
                | "trzeci"
                | "trzecie"
                | "czwarta"
                | "czwarty"
                | "piata"
                | "piaty"
                | "szosta"
                | "szosty"
                | "siodma"
                | "siodmy"
                | "osma"
                | "osmy"
                | "dziewiata"
                | "dziewiaty"
                | "dziesiata"
                | "dziesiaty"
                | "jedenasta"
                | "jedenasty"
                | "dwunasta"
                | "dwunasty"
                | "trzynasta"
                | "trzynasty"
                | "czternasta"
                | "czternasty"
                | "pietnasta"
                | "pietnasty"
                | "szesnasta"
                | "szesnasty"
                | "siedemnasta"
                | "siedemnasty"
                | "osiemnasta"
                | "osiemnasty"
                | "dziewietnasta"
                | "dziewietnasty"
                | "dwudziesta"
                | "dwudziesty"
                | "trzydziesta"
                | "trzydziesty"
                | "czterdziesta"
                | "czterdziesty"
                | "piecdziesiata"
                | "piecdziesiaty"
                | "jeden"
                | "dwa"
                | "trzy"
                | "cztery"
                | "piec"
                | "szesc"
                | "siedem"
                | "osiem"
                | "dziewiec"
                | "dziesiec"
                | "jedenascie"
                | "dwanascie"
                | "trzynascie"
                | "czternascie"
                | "pietnascie"
                | "szesnascie"
                | "siedemnascie"
                | "osiemnascie"
                | "dziewietnascie"
                | "dwadziescia"
        )
}

fn detect_silence_points(
    ffmpeg: &Path,
    file: &Path,
    duration: f64,
    resource: ResourceProfile,
    cancel: &AtomicBool,
    hwnd: Hwnd,
    progress_start: u32,
    progress_end: u32,
) -> Result<(Vec<SilencePoint>, f64), String> {
    let mut command = Command::new(ffmpeg);
    command
        .arg("-hide_banner")
        .arg("-nostats")
        .arg("-nostdin")
        .arg("-i")
        .arg(file)
        .arg("-af")
        .arg("highpass=f=70,lowpass=f=9000,afftdn=nr=12:nf=-35:tn=1,silencedetect=noise=-38dB:d=1.2")
        .arg("-progress")
        .arg("pipe:1")
        .arg("-stats_period")
        .arg("1")
        .arg("-f")
        .arg("null")
        .arg("NUL");

    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .creation_flags(CREATE_NO_WINDOW);
    let mut child = command
        .spawn()
        .map_err(|err| format!("nie można uruchomić FFmpeg: {}", err))?;
    set_priority(child.id(), resource);
    let stdout = child.stdout.take().ok_or_else(|| "brak danych postępu FFmpeg".to_string())?;
    let stderr = child.stderr.take().ok_or_else(|| "brak dziennika FFmpeg".to_string())?;
    let progress_reader = thread::spawn(move || {
        read_ffmpeg_progress(stdout, duration, hwnd, progress_start, progress_end)
    });
    let reader = thread::spawn(move || {
        let mut input = BufReader::new(stderr);
        let mut line = Vec::new();
        let mut points = Vec::new();
        let mut last_start: Option<f64> = None;
        let mut tail = Vec::new();
        loop {
            line.clear();
            match input.read_until(b'\n', &mut line) {
                Ok(0) => break,
                Ok(_) => {
                    push_tail(&mut tail, &line, 96 * 1024);
                    let text = String::from_utf8_lossy(&line);
                    if let Some(value) = number_after(&text, "silence_start:") {
                        last_start = Some(value);
                    }
                    if let Some(end) = number_after(&text, "silence_end:") {
                        let start = last_start.take().unwrap_or(end);
                        let point = SilencePoint { start, end };
                        let midpoint = point.midpoint();
                        if point.duration() >= 1.15
                            && midpoint > 5.0
                            && midpoint < duration - 5.0
                        {
                            points.push(point);
                        }
                    }
                }
                Err(err) => return Err(format!("błąd odczytu dziennika FFmpeg: {}", err)),
            }
        }
        Ok((points, tail))
    });

    let status = wait_for_child(child, cancel, hwnd, "Wykrywanie ciszy");
    let (mut points, tail) = reader
        .join()
        .map_err(|_| "błąd wątku odczytującego FFmpeg".to_string())??;
    let scanned_duration = progress_reader
        .join()
        .map_err(|_| "błąd wątku postępu FFmpeg".to_string())??;
    let status = status?;
    if !status.success() {
        return Err(short_stderr_error("ffmpeg silencedetect", &tail));
    }
    points.sort_by(|a, b| {
        a.midpoint()
            .partial_cmp(&b.midpoint())
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    post_progress(hwnd, progress_end);
    Ok((points, scanned_duration.max(duration)))
}

fn build_segments(duration: f64, segment_seconds: f64, silence_points: &[f64]) -> Vec<(f64, f64)> {
    let mut boundaries = Vec::new();
    let mut last = 0.0;
    while last + segment_seconds < duration - 1.0 {
        let desired = last + segment_seconds;
        let window = (segment_seconds * 0.15).clamp(30.0, 180.0);
        let lower = (desired - window).max(last + segment_seconds * 0.55);
        let upper = (desired + window).min(duration - 2.0);
        let boundary = silence_points
            .iter()
            .copied()
            .filter(|point| *point >= lower && *point <= upper)
            .min_by(|a, b| {
                (a - desired)
                    .abs()
                    .partial_cmp(&(b - desired).abs())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap_or(desired.min(duration));

        if boundary <= last + 1.0 {
            break;
        }
        boundaries.push(boundary);
        last = boundary;
    }

    if let Some(last_boundary) = boundaries.last().copied() {
        let tail_threshold = (segment_seconds * 0.25).clamp(60.0, 600.0);
        let tiny_tail = (duration - last_boundary) < tail_threshold;
        if tiny_tail {
            boundaries.pop();
        }
    }

    let mut segments = Vec::new();
    let mut start = 0.0;
    for boundary in boundaries {
        if boundary > start + 1.0 {
            segments.push((start, boundary));
            start = boundary;
        }
    }
    if duration > start + 1.0 {
        segments.push((start, duration));
    }
    segments
}

fn build_audiobook_segments(
    duration: f64,
    maximum_seconds: f64,
    chapter_boundaries: &[f64],
    silence_points: &[f64],
) -> Vec<(f64, f64)> {
    let mut chapters: Vec<f64> = chapter_boundaries
        .iter()
        .copied()
        .filter(|point| point.is_finite() && *point > 5.0 && *point < duration - 5.0)
        .collect();
    chapters.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    chapters.dedup_by(|a, b| (*a - *b).abs() < 2.0);

    let mut boundaries = Vec::new();
    let mut anchors = Vec::with_capacity(chapters.len() + 2);
    anchors.push(0.0);
    anchors.extend(chapters);
    anchors.push(duration);

    for pair in anchors.windows(2) {
        let interval_start = pair[0];
        let interval_end = pair[1];
        let mut cursor = interval_start;
        while cursor + maximum_seconds < interval_end - 1.0 {
            let desired = cursor + maximum_seconds;
            let small_tail = interval_end - desired
                < (maximum_seconds * 0.25).clamp(60.0, 600.0);
            if small_tail {
                break;
            }
            let window = (maximum_seconds * 0.15).clamp(30.0, 180.0);
            let lower = (desired - window).max(cursor + maximum_seconds * 0.55);
            let upper = (desired + window).min(interval_end - 2.0);
            let boundary = silence_points
                .iter()
                .copied()
                .filter(|point| *point >= lower && *point <= upper)
                .min_by(|a, b| {
                    (a - desired)
                        .abs()
                        .partial_cmp(&(b - desired).abs())
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .unwrap_or(desired);
            if boundary <= cursor + 1.0 {
                break;
            }
            boundaries.push(boundary);
            cursor = boundary;
        }
        if interval_end < duration - 1.0 {
            boundaries.push(interval_end);
        }
    }

    boundaries.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    boundaries.dedup_by(|a, b| (*a - *b).abs() < 1.0);
    let mut segments = Vec::new();
    let mut start = 0.0;
    for boundary in boundaries {
        if boundary > start + 1.0 {
            segments.push((start, boundary));
            start = boundary;
        }
    }
    if duration > start + 1.0 {
        segments.push((start, duration));
    }
    segments
}

fn cut_segment(
    ffmpeg: &Path,
    input: &Path,
    output: &Path,
    start: f64,
    duration: f64,
    format: OutputFormat,
    resource: ResourceProfile,
    bitrate_kbps: u32,
    sample_rate: Option<u32>,
    cancel: &AtomicBool,
    hwnd: Hwnd,
    progress_start: u32,
    progress_end: u32,
) -> Result<(), String> {
    let mut command = Command::new(ffmpeg);
    command
        .arg("-hide_banner")
        .arg("-nostats")
        .arg("-y")
        .arg("-nostdin")
        .arg("-progress")
        .arg("pipe:1")
        .arg("-stats_period")
        .arg("1")
        .arg("-ss")
        .arg(seconds_arg(start))
        .arg("-i")
        .arg(input)
        .arg("-t")
        .arg(seconds_arg(duration))
        .arg("-map")
        .arg("0:a:0")
        .arg("-vn");

    match format {
        OutputFormat::Same => {
            command.arg("-c").arg("copy").arg("-avoid_negative_ts").arg("make_zero");
        }
        OutputFormat::Mp3 => {
            command
                .arg("-c:a")
                .arg("libmp3lame")
                .arg("-b:a")
                .arg(format!("{}k", bitrate_kbps));
            command.arg("-threads").arg(resource.thread_count().to_string());
        }
        OutputFormat::M4a => {
            command
                .arg("-c:a")
                .arg("aac")
                .arg("-b:a")
                .arg(format!("{}k", bitrate_kbps));
            command.arg("-threads").arg(resource.thread_count().to_string());
        }
        OutputFormat::Wav => {
            command.arg("-c:a").arg("pcm_s16le");
        }
        OutputFormat::Flac => {
            command.arg("-c:a").arg("flac").arg("-compression_level").arg("5");
            command.arg("-threads").arg(resource.thread_count().to_string());
        }
        OutputFormat::Opus => {
            command
                .arg("-c:a")
                .arg("libopus")
                .arg("-b:a")
                .arg(format!("{}k", bitrate_kbps));
            command.arg("-threads").arg(resource.thread_count().to_string());
        }
    }
    if !matches!(format, OutputFormat::Same) {
        if let Some(sample_rate) = sample_rate {
            command.arg("-ar").arg(sample_rate.to_string());
        }
    }
    command.arg(output);

    let result = run_ffmpeg(
        command,
        resource,
        cancel,
        hwnd,
        "Cięcie nadal trwa",
        Some((duration, progress_start, progress_end)),
    )?;
    if result.status.success() {
        Ok(())
    } else {
        Err(short_stderr_error("ffmpeg", &result.stderr))
    }
}

struct ProcessReport {
    status: ExitStatus,
    stderr: Vec<u8>,
}

fn run_ffmpeg(
    mut command: Command,
    resource: ResourceProfile,
    cancel: &AtomicBool,
    hwnd: Hwnd,
    activity: &str,
    progress: Option<(f64, u32, u32)>,
) -> Result<ProcessReport, String> {
    command
        .stdin(Stdio::null())
        .stdout(if progress.is_some() { Stdio::piped() } else { Stdio::null() })
        .stderr(Stdio::piped())
        .creation_flags(CREATE_NO_WINDOW);

    let mut child = command
        .spawn()
        .map_err(|err| format!("nie można uruchomić narzędzia: {}", err))?;
    set_priority(child.id(), resource);
    let progress_reader = if let Some((duration, start, end)) = progress {
        let stdout = child.stdout.take().ok_or_else(|| "brak danych postępu procesu".to_string())?;
        Some(thread::spawn(move || {
            read_ffmpeg_progress(stdout, duration, hwnd, start, end)
        }))
    } else {
        None
    };
    let stderr = child.stderr.take().ok_or_else(|| "brak dziennika procesu".to_string())?;
    let reader = thread::spawn(move || read_tail(stderr, 96 * 1024));
    let status = wait_for_child(child, cancel, hwnd, activity);
    let stderr = reader
        .join()
        .map_err(|_| "błąd wątku odczytującego dziennik procesu".to_string())??;
    if let Some(reader) = progress_reader {
        reader
            .join()
            .map_err(|_| "błąd wątku postępu procesu".to_string())??;
    }
    let status = status?;
    if status.success() {
        if let Some((_, _, end)) = progress {
            post_progress(hwnd, end);
        }
    }
    Ok(ProcessReport {
        status,
        stderr,
    })
}

fn read_ffmpeg_progress<R: Read>(
    input: R,
    duration: f64,
    hwnd: Hwnd,
    progress_start: u32,
    progress_end: u32,
) -> Result<f64, String> {
    let mut reader = BufReader::new(input);
    let mut line = String::new();
    let mut last_position = progress_start;
    let mut scanned_seconds = 0.0f64;
    post_progress(hwnd, progress_start);
    loop {
        line.clear();
        let count = reader
            .read_line(&mut line)
            .map_err(|err| format!("błąd odczytu postępu FFmpeg: {}", err))?;
        if count == 0 {
            break;
        }
        if let Some(value) = line.strip_prefix("out_time_us=") {
            if let Ok(microseconds) = value.trim().parse::<f64>() {
                scanned_seconds = scanned_seconds.max(microseconds / 1_000_000.0);
                let ratio = if duration > 0.0 {
                    (microseconds / 1_000_000.0 / duration).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                let position = progress_start
                    + ((progress_end - progress_start) as f64 * ratio).round() as u32;
                if position != last_position {
                    post_progress(hwnd, position);
                    last_position = position;
                }
            }
        }
    }
    Ok(scanned_seconds)
}

fn wait_for_child(
    mut child: std::process::Child,
    cancel: &AtomicBool,
    hwnd: Hwnd,
    activity: &str,
) -> Result<ExitStatus, String> {
    let mut last_message = Instant::now();
    loop {
        if cancel.load(Ordering::SeqCst) {
            let _ = child.kill();
            let _ = child.wait();
            return Err("przetwarzanie zostało zatrzymane przez użytkownika".to_string());
        }
        match child.try_wait() {
            Ok(Some(status)) => return Ok(status),
            Ok(None) => {}
            Err(err) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("błąd oczekiwania na proces: {}", err));
            }
        }
        if last_message.elapsed() >= Duration::from_secs(60) {
            post_log(hwnd, &format!("{} — program działa, proszę czekać...", activity));
            last_message = Instant::now();
        }
        thread::sleep(Duration::from_millis(200));
    }
}

fn read_tail<R: Read>(mut input: R, limit: usize) -> Result<Vec<u8>, String> {
    let mut result = Vec::new();
    let mut buffer = [0u8; 8192];
    loop {
        match input.read(&mut buffer) {
            Ok(0) => return Ok(result),
            Ok(count) => push_tail(&mut result, &buffer[..count], limit),
            Err(err) => return Err(format!("błąd odczytu dziennika procesu: {}", err)),
        }
    }
}

fn push_tail(target: &mut Vec<u8>, bytes: &[u8], limit: usize) {
    target.extend_from_slice(bytes);
    if target.len() > limit {
        let excess = target.len() - limit;
        target.drain(..excess);
    }
}

fn run_capture(mut command: Command, resource: ResourceProfile) -> Result<Output, String> {
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .creation_flags(CREATE_NO_WINDOW);

    let child = command.spawn().map_err(|err| format!("nie można uruchomić narzędzia: {}", err))?;
    set_priority(child.id(), resource);
    child.wait_with_output().map_err(|err| format!("błąd oczekiwania na proces: {}", err))
}

fn set_priority(process_id: u32, resource: ResourceProfile) {
    unsafe {
        let handle = OpenProcess(PROCESS_SET_INFORMATION, 0, process_id);
        if handle == 0 {
            return;
        }
        let class = match resource {
            ResourceProfile::Low => IDLE_PRIORITY_CLASS,
            ResourceProfile::Normal => BELOW_NORMAL_PRIORITY_CLASS,
            ResourceProfile::Fast => NORMAL_PRIORITY_CLASS,
        };
        SetPriorityClass(handle, class);
        CloseHandle(handle);
    }
}

impl ResourceProfile {
    fn thread_count(self) -> usize {
        let cpus = thread::available_parallelism().map(|value| value.get()).unwrap_or(4);
        match self {
            ResourceProfile::Low => 1,
            ResourceProfile::Normal => (cpus / 2).clamp(1, 4),
            ResourceProfile::Fast => cpus.clamp(1, 16),
        }
    }
}

fn output_directory(input: &Path, selected_root: Option<&Path>) -> Result<PathBuf, String> {
    let stem = input
        .file_stem()
        .and_then(|value| value.to_str())
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("audio");
    let folder_name = format!("{}_{}", stem, tr("pocięte", "split", "geteilt", "разделено"));
    if let Some(root) = selected_root {
        Ok(root.join(folder_name))
    } else {
        let parent = input.parent().ok_or_else(|| "plik nie ma folderu nadrzędnego".to_string())?;
        Ok(parent.join(folder_name))
    }
}

fn segment_path(input: &Path, output_dir: &Path, index: usize, format: OutputFormat) -> PathBuf {
    let stem = input.file_stem().and_then(|value| value.to_str()).unwrap_or("audio");
    let ext = match format {
        OutputFormat::Same => input.extension().and_then(|value| value.to_str()).unwrap_or("mka"),
        OutputFormat::Mp3 => "mp3",
        OutputFormat::M4a => "m4a",
        OutputFormat::Wav => "wav",
        OutputFormat::Flac => "flac",
        OutputFormat::Opus => "opus",
    };
    output_dir.join(format!("{}_{}_{:03}.{}", stem, tr("część", "part", "Teil", "часть"), index, ext))
}

fn executable_directory() -> Result<PathBuf, String> {
    std::env::current_exe()
        .map_err(|error| format!("nie można ustalić położenia programu: {}", error))?
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| "plik programu nie ma katalogu nadrzędnego".to_string())
}

fn chapter_detector_root() -> Result<PathBuf, String> {
    Ok(executable_directory()?
        .join("tools")
        .join("chapter-detector"))
}

fn chapter_detector_components_present(prefer_nvidia: bool) -> bool {
    let Ok(root) = chapter_detector_root() else {
        return false;
    };
    let model = root.join("models").join(CHAPTER_MODEL_FILE);
    let cpu = find_file_recursive(&root.join("cpu"), "whisper-cli.exe").is_some();
    let cuda = !prefer_nvidia
        || find_file_recursive(&root.join("cuda"), "whisper-cli.exe").is_some();
    model.is_file() && cpu && cuda
}

fn nvidia_available() -> bool {
    let mut command = Command::new("nvidia-smi.exe");
    command
        .arg("-L")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .creation_flags(CREATE_NO_WINDOW);
    command.status().is_ok_and(|status| status.success())
}

fn parse_chapter_detector_config(text: &str) -> Result<ChapterDetectorConfig, String> {
    let value = |key: &str| -> Result<String, String> {
        text.lines()
            .filter_map(|line| line.trim().split_once('='))
            .find(|(name, _)| name.trim() == key)
            .map(|(_, value)| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .ok_or_else(|| format!("konfiguracja modelu nie zawiera pola {}", key))
    };
    let config = ChapterDetectorConfig {
        cpu_url: value("cpu_url")?,
        cpu_sha256: value("cpu_sha256")?,
        cuda_url: value("cuda_url")?,
        cuda_sha256: value("cuda_sha256")?,
        model_url: value("model_url")?,
        model_sha256: value("model_sha256")?,
        model_size: value("model_size")?
            .parse::<u64>()
            .map_err(|_| "konfiguracja modelu ma nieprawidłowy rozmiar".to_string())?,
    };
    validate_sha256(&config.cpu_sha256)?;
    validate_sha256(&config.cuda_sha256)?;
    validate_sha256(&config.model_sha256)?;
    validate_chapter_download_url(&config.cpu_url, false)?;
    validate_chapter_download_url(&config.cuda_url, false)?;
    validate_chapter_download_url(&config.model_url, true)?;
    if config.model_size < 50_000_000 || config.model_size > 1_000_000_000 {
        return Err("konfiguracja modelu ma niedozwolony rozmiar".to_string());
    }
    Ok(config)
}

fn validate_chapter_download_url(url: &str, model: bool) -> Result<(), String> {
    let valid = if model {
        url.starts_with("https://huggingface.co/ggerganov/whisper.cpp/resolve/")
            && url.ends_with(".bin")
    } else {
        url.starts_with(
            "https://github.com/ggml-org/whisper.cpp/releases/download/",
        ) && url.ends_with(".zip")
    };
    if valid && !url.bytes().any(|byte| byte.is_ascii_whitespace()) {
        Ok(())
    } else {
        Err("serwer zwrócił niedozwolony adres składnika rozpoznawania".to_string())
    }
}

fn ensure_chapter_detector(
    hwnd: Hwnd,
    prefer_nvidia: bool,
    cancel: &AtomicBool,
) -> Result<ChapterDetector, String> {
    post_log(
        hwnd,
        "Przygotowuję lokalne rozpoznawanie rozdziałów audiobooka.",
    );
    let config_text = read_static_text(CHAPTER_CONFIG_FILE, "konfiguracji modelu")?;
    let config = parse_chapter_detector_config(&config_text)?;
    let root = chapter_detector_root()?;
    fs::create_dir_all(&root)
        .map_err(|error| format!("nie można utworzyć katalogu modelu: {}", error))?;

    let cpu_directory = root.join("cpu");
    let cpu_exe = ensure_whisper_engine(
        hwnd,
        &root,
        &cpu_directory,
        "CPU",
        &config.cpu_url,
        &config.cpu_sha256,
        cancel,
    )?;

    let model_directory = root.join("models");
    fs::create_dir_all(&model_directory)
        .map_err(|error| format!("nie można utworzyć katalogu modeli: {}", error))?;
    let model = model_directory.join(CHAPTER_MODEL_FILE);
    ensure_whisper_model(hwnd, &model, &config, cancel)?;

    let mut primary_exe = cpu_exe.clone();
    let mut using_nvidia = false;
    if prefer_nvidia {
        let cuda_directory = root.join("cuda");
        match ensure_whisper_engine(
            hwnd,
            &root,
            &cuda_directory,
            "NVIDIA CUDA",
            &config.cuda_url,
            &config.cuda_sha256,
            cancel,
        ) {
            Ok(cuda_exe) => match verify_nvidia_engine(&cuda_exe) {
                Ok(device) => {
                    primary_exe = cuda_exe;
                    using_nvidia = true;
                    post_log(
                        hwnd,
                        &format!(
                            "Silnik CUDA potwierdził aktywne przyspieszenie NVIDIA: {}.",
                            device
                        ),
                    );
                }
                Err(error) => {
                    post_log(
                        hwnd,
                        &format!(
                            "Silnik CUDA nie uruchomił backendu NVIDIA. Używam CPU: {}",
                            error
                        ),
                    );
                }
            }
            Err(error) if cancel.load(Ordering::SeqCst) => return Err(error),
            Err(error) => {
                post_log(
                    hwnd,
                    &format!(
                        "Nie udało się przygotować przyspieszenia NVIDIA. Używam CPU: {}",
                        error
                    ),
                );
            }
        }
    } else {
        post_log(hwnd, "Rozpoznawanie będzie działało na CPU.");
    }

    Ok(ChapterDetector {
        cpu_exe,
        primary_exe,
        model,
        using_nvidia,
    })
}

fn verify_nvidia_engine(executable: &Path) -> Result<String, String> {
    let mut command = Command::new(executable);
    command.arg("--version");
    let output = run_capture(command, ResourceProfile::Normal)?;
    if !output.status.success() {
        return Err(short_process_error("whisper-cli CUDA", &output));
    }
    let details = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout)
    );
    if !details.contains("loaded CUDA backend") {
        return Err("paczka nie zgłosiła załadowanego backendu CUDA".to_string());
    }
    details
        .lines()
        .map(str::trim)
        .find(|line| line.starts_with("Device "))
        .map(|line| line.to_string())
        .ok_or_else(|| "backend CUDA nie zgłosił żadnej karty NVIDIA".to_string())
}

fn ensure_whisper_model(
    hwnd: Hwnd,
    model: &Path,
    config: &ChapterDetectorConfig,
    cancel: &AtomicBool,
) -> Result<(), String> {
    if model.is_file()
        && fs::metadata(model)
            .map(|metadata| metadata.len() == config.model_size)
            .unwrap_or(false)
        && verify_sha256(model, &config.model_sha256).is_ok()
    {
        post_log(hwnd, "Lokalny model Whisper jest gotowy.");
        return Ok(());
    }
    if model.exists() {
        fs::remove_file(model)
            .map_err(|error| format!("nie można usunąć uszkodzonego modelu: {}", error))?;
    }
    let temporary = model.with_extension("bin.part");
    let _ = fs::remove_file(&temporary);
    post_log(
        hwnd,
        "Pobieram wielojęzyczny model Whisper small q5 do folderu programu (około 181 MB)...",
    );
    download_url_cancellable(
        &config.model_url,
        &temporary,
        cancel,
        hwnd,
        "Pobieranie modelu rozpoznawania",
    )
        .map_err(|error| format!("nie można pobrać modelu Whisper: {}", error))?;
    let size = fs::metadata(&temporary)
        .map_err(|error| format!("nie można sprawdzić rozmiaru modelu: {}", error))?
        .len();
    if size != config.model_size {
        let _ = fs::remove_file(&temporary);
        return Err(format!(
            "pobrany model ma nieprawidłowy rozmiar: {} zamiast {} bajtów",
            size, config.model_size
        ));
    }
    verify_sha256(&temporary, &config.model_sha256).map_err(|error| {
        let _ = fs::remove_file(&temporary);
        format!("model Whisper nie przeszedł weryfikacji: {}", error)
    })?;
    fs::rename(&temporary, model)
        .map_err(|error| format!("nie można zapisać gotowego modelu: {}", error))?;
    post_log(hwnd, "Model Whisper został pobrany i zweryfikowany.");
    Ok(())
}

fn ensure_whisper_engine(
    hwnd: Hwnd,
    root: &Path,
    directory: &Path,
    description: &str,
    url: &str,
    expected_sha256: &str,
    cancel: &AtomicBool,
) -> Result<PathBuf, String> {
    let marker = directory.join(".package-sha256");
    if fs::read_to_string(&marker)
        .map(|value| value.trim().eq_ignore_ascii_case(expected_sha256))
        .unwrap_or(false)
    {
        if let Some(executable) = find_file_recursive(directory, "whisper-cli.exe") {
            post_log(hwnd, &format!("Silnik {} jest gotowy.", description));
            return Ok(executable);
        }
    }

    let safe_name = if description.contains("NVIDIA") {
        "cuda"
    } else {
        "cpu"
    };
    let archive = root.join(format!(".whisper-{}-download.zip", safe_name));
    let extract = root.join(format!(".whisper-{}-extract", safe_name));
    let _ = fs::remove_file(&archive);
    if extract.exists() {
        fs::remove_dir_all(&extract)
            .map_err(|error| format!("nie można wyczyścić katalogu roboczego: {}", error))?;
    }
    fs::create_dir_all(&extract)
        .map_err(|error| format!("nie można utworzyć katalogu roboczego: {}", error))?;
    post_log(
        hwnd,
        &format!("Pobieram silnik rozpoznawania {}...", description),
    );
    download_url_cancellable(
        url,
        &archive,
        cancel,
        hwnd,
        &format!("Pobieranie silnika {}", description),
    )
        .map_err(|error| format!("nie można pobrać silnika {}: {}", description, error))?;
    verify_sha256(&archive, expected_sha256).map_err(|error| {
        let _ = fs::remove_file(&archive);
        format!("silnik {} nie przeszedł weryfikacji: {}", description, error)
    })?;
    extract_zip_archive(&archive, &extract)?;
    let _ = fs::remove_file(&archive);
    if find_file_recursive(&extract, "whisper-cli.exe").is_none() {
        let _ = fs::remove_dir_all(&extract);
        return Err(format!(
            "paczka silnika {} nie zawiera whisper-cli.exe",
            description
        ));
    }
    if directory.exists() {
        fs::remove_dir_all(directory)
            .map_err(|error| format!("nie można zastąpić silnika {}: {}", description, error))?;
    }
    fs::rename(&extract, directory)
        .map_err(|error| format!("nie można zainstalować silnika {}: {}", description, error))?;
    fs::write(&marker, expected_sha256)
        .map_err(|error| format!("nie można zapisać informacji o silniku: {}", error))?;
    let executable = find_file_recursive(directory, "whisper-cli.exe")
        .ok_or_else(|| format!("nie odnaleziono silnika {} po instalacji", description))?;
    post_log(
        hwnd,
        &format!("Silnik {} został pobrany i zweryfikowany.", description),
    );
    Ok(executable)
}

fn extract_zip_archive(archive: &Path, destination: &Path) -> Result<(), String> {
    let script =
        "Expand-Archive -LiteralPath $env:MAJHER_ARCHIVE_PATH -DestinationPath $env:MAJHER_EXTRACT_PATH -Force";
    let mut command = Command::new("powershell.exe");
    command
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            script,
        ])
        .env("MAJHER_ARCHIVE_PATH", archive)
        .env("MAJHER_EXTRACT_PATH", destination)
        .creation_flags(CREATE_NO_WINDOW);
    let output = command
        .output()
        .map_err(|error| format!("nie można uruchomić rozpakowywania: {}", error))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "rozpakowywanie nie powiodło się: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ))
    }
}

fn find_on_path(name: &str) -> Option<PathBuf> {
    let mut command = Command::new("where.exe");
    command
        .arg(name)
        .creation_flags(CREATE_NO_WINDOW);
    let output = command.output().ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(PathBuf::from)
        .filter(|path| path.is_file())
}

fn find_system_tools() -> Option<(PathBuf, PathBuf)> {
    Some((
        find_on_path("ffmpeg.exe")?,
        find_on_path("ffprobe.exe")?,
    ))
}

fn find_local_tools() -> Option<(PathBuf, PathBuf)> {
    let exe_dir = executable_directory().ok()?;
    for directory in [
        exe_dir.join("tools").join("ffmpeg"),
        exe_dir.join("tools"),
        exe_dir,
    ] {
        let ffmpeg = directory.join("ffmpeg.exe");
        let ffprobe = directory.join("ffprobe.exe");
        if ffmpeg.is_file() && ffprobe.is_file() {
            return Some((ffmpeg, ffprobe));
        }
    }
    None
}

fn ensure_ffmpeg(hwnd: Hwnd) -> Result<(PathBuf, PathBuf), String> {
    if let Some(tools) = find_system_tools() {
        post_log(hwnd, "Znaleziono FFmpeg zainstalowany w systemie.");
        show_message_on_ui(
            hwnd,
            "Wykryto FFmpeg i FFprobe zainstalowane w systemie.\n\nMajher użyje wersji systemowej i nie będzie niczego pobierać.",
        );
        return Ok(tools);
    }
    if let Some(tools) = find_local_tools() {
        post_log(hwnd, "Korzystam z lokalnego FFmpeg w katalogu programu.");
        return Ok(tools);
    }

    post_log(
        hwnd,
        "FFmpeg nie jest dostępny w systemie. Tworzę katalog tools i pobieram potrzebne pliki...",
    );
    let exe_dir = executable_directory()?;
    let tools_root = exe_dir.join("tools");
    let ffmpeg_dir = tools_root.join("ffmpeg");
    fs::create_dir_all(&ffmpeg_dir)
        .map_err(|error| format!("nie można utworzyć katalogu {}: {}", ffmpeg_dir.display(), error))?;

    let ffmpeg_url = read_static_text("ffmpeg-url.txt", "adresu FFmpeg")?;
    let ffmpeg_url = ffmpeg_url.trim();
    if !ffmpeg_url.starts_with("https://www.gyan.dev/ffmpeg/builds/")
        || !ffmpeg_url.ends_with(".zip")
        || ffmpeg_url.bytes().any(|byte| byte.is_ascii_whitespace())
    {
        return Err("serwer zwrócił nieprawidłowy adres paczki FFmpeg".to_string());
    }

    let archive = tools_root.join(".ffmpeg-download.zip");
    let checksum_file = tools_root.join(".ffmpeg-download.sha256");
    let extract_dir = tools_root.join(".ffmpeg-extract");
    let _ = fs::remove_file(&archive);
    let _ = fs::remove_file(&checksum_file);
    if extract_dir.exists() {
        fs::remove_dir_all(&extract_dir)
            .map_err(|error| format!("nie można wyczyścić katalogu roboczego FFmpeg: {}", error))?;
    }

    download_url(
        &format!("{}.sha256?t={}", ffmpeg_url, cache_buster()),
        &checksum_file,
    )?;
    let expected_sha256 = fs::read_to_string(&checksum_file)
        .map_err(|error| format!("nie można odczytać sumy FFmpeg: {}", error))?;
    let _ = fs::remove_file(&checksum_file);
    let expected_sha256 = expected_sha256.trim();
    validate_sha256(expected_sha256)?;

    download_url(
        &format!("{}?t={}", ffmpeg_url, cache_buster()),
        &archive,
    )?;
    verify_sha256(&archive, expected_sha256).map_err(|error| {
        let _ = fs::remove_file(&archive);
        format!("paczka FFmpeg nie przeszła weryfikacji: {}", error)
    })?;

    post_log(hwnd, "Pobrano FFmpeg. Rozpakowuję paczkę...");
    fs::create_dir_all(&extract_dir)
        .map_err(|error| format!("nie można utworzyć katalogu roboczego FFmpeg: {}", error))?;
    let script = "Expand-Archive -LiteralPath $env:MAJHER_ARCHIVE_PATH -DestinationPath $env:MAJHER_FFMPEG_DIR -Force";
    let mut command = Command::new("powershell.exe");
    command
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            script,
        ])
        .env("MAJHER_ARCHIVE_PATH", &archive)
        .env("MAJHER_FFMPEG_DIR", &extract_dir)
        .creation_flags(CREATE_NO_WINDOW);
    let output = command
        .output()
        .map_err(|error| format!("nie można uruchomić rozpakowywania: {}", error))?;
    let _ = fs::remove_file(&archive);
    if !output.status.success() {
        let details = String::from_utf8_lossy(&output.stderr);
        return Err(format!("rozpakowanie FFmpeg nie powiodło się: {}", details.trim()));
    }

    let extracted_ffmpeg = find_file_recursive(&extract_dir, "ffmpeg.exe")
        .ok_or_else(|| "najnowsza paczka nie zawiera ffmpeg.exe".to_string())?;
    let extracted_ffprobe = find_file_recursive(&extract_dir, "ffprobe.exe")
        .ok_or_else(|| "najnowsza paczka nie zawiera ffprobe.exe".to_string())?;
    let ffmpeg = ffmpeg_dir.join("ffmpeg.exe");
    let ffprobe = ffmpeg_dir.join("ffprobe.exe");
    fs::copy(&extracted_ffmpeg, &ffmpeg)
        .map_err(|error| format!("nie można zapisać ffmpeg.exe: {}", error))?;
    fs::copy(&extracted_ffprobe, &ffprobe)
        .map_err(|error| format!("nie można zapisać ffprobe.exe: {}", error))?;
    if let Some(license) = find_file_recursive(&extract_dir, "LICENSE") {
        let _ = fs::copy(license, ffmpeg_dir.join("FFmpeg-LICENSE.txt"));
    }
    let _ = fs::remove_dir_all(&extract_dir);
    post_log(hwnd, "FFmpeg jest gotowy w katalogu tools\\ffmpeg.");
    Ok((ffmpeg, ffprobe))
}

fn find_file_recursive(root: &Path, name: &str) -> Option<PathBuf> {
    let entries = fs::read_dir(root).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file()
            && path
                .file_name()
                .is_some_and(|file_name| file_name.eq_ignore_ascii_case(name))
        {
            return Some(path);
        }
        if path.is_dir() {
            if let Some(found) = find_file_recursive(&path, name) {
                return Some(found);
            }
        }
    }
    None
}

#[inline(never)]
fn decode_hidden(value: &[u8]) -> String {
    let key = std::hint::black_box(OBFUSCATION_KEY);
    String::from_utf8(value.iter().map(|byte| byte ^ key).collect())
        .unwrap_or_default()
}

fn cache_buster() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

fn static_url(file: &str) -> Result<String, String> {
    validate_download_name(file)?;
    Ok(format!(
        "{}{}?t={}",
        decode_hidden(UPDATE_BASE_ENCODED),
        file,
        cache_buster()
    ))
}

fn validate_download_name(value: &str) -> Result<(), String> {
    if value.is_empty()
        || value.len() > 40
        || !value
            .bytes()
            .all(|byte| {
                byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_' || byte == b'.'
            })
    {
        Err("serwer zwrócił nieprawidłową nazwę pliku".to_string())
    } else {
        Ok(())
    }
}

fn download_url(url: &str, destination: &Path) -> Result<(), String> {
    let url = wide(url);
    let destination_wide = wide(&destination.to_string_lossy());
    let result = unsafe {
        URLDownloadToFileW(
            null_mut(),
            url.as_ptr(),
            destination_wide.as_ptr(),
            0,
            null_mut(),
        )
    };
    if result < 0 {
        let code = result as u32;
        let explanation = match code {
            0x800C0006 => "nie znaleziono pliku pod wskazanym adresem",
            0x800C0005 => "serwer odrzucił lub przerwał połączenie",
            _ => "serwer lub połączenie HTTPS zwróciło błąd",
        };
        Err(format!("{} (0x{:08X})", explanation, code))
    } else if !destination.is_file() {
        Err("pobieranie zakończyło się bez utworzenia pliku".to_string())
    } else {
        Ok(())
    }
}

fn download_url_cancellable(
    url: &str,
    destination: &Path,
    cancel: &AtomicBool,
    hwnd: Hwnd,
    activity: &str,
) -> Result<(), String> {
    let script = "$ProgressPreference='SilentlyContinue'; Invoke-WebRequest -UseBasicParsing -Uri $env:MAJHER_DOWNLOAD_URL -OutFile $env:MAJHER_DOWNLOAD_PATH";
    let mut command = Command::new("powershell.exe");
    command
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            script,
        ])
        .env("MAJHER_DOWNLOAD_URL", url)
        .env("MAJHER_DOWNLOAD_PATH", destination);
    let report = run_ffmpeg(
        command,
        ResourceProfile::Normal,
        cancel,
        hwnd,
        activity,
        None,
    )?;
    if !report.status.success() {
        return Err(short_stderr_error("pobieranie HTTPS", &report.stderr));
    }
    if destination.is_file() {
        Ok(())
    } else {
        Err("pobieranie zakończyło się bez utworzenia pliku".to_string())
    }
}

fn download_static_file(file: &str, destination: &Path) -> Result<(), String> {
    download_url(&static_url(file)?, destination)
        .map_err(|error| format!("nie można pobrać pliku {}: {}", file, error))
}

fn read_static_text(file: &str, description: &str) -> Result<String, String> {
    validate_download_name(file)?;
    let temporary = std::env::temp_dir().join(format!(
        "majher-{}-{}.tmp",
        std::process::id(),
        file.replace('.', "-")
    ));
    let _ = fs::remove_file(&temporary);
    download_static_file(file, &temporary)?;
    let contents = fs::read_to_string(&temporary)
        .map_err(|error| format!("nie można odczytać {}: {}", description, error));
    let _ = fs::remove_file(&temporary);
    contents
}

fn version_parts(version: &str) -> Option<Vec<u64>> {
    if version.is_empty() {
        return None;
    }
    version
        .split('.')
        .map(|part| part.parse::<u64>().ok())
        .collect()
}

fn version_is_newer(remote: &str, current: &str) -> Result<bool, String> {
    let mut remote = version_parts(remote)
        .ok_or_else(|| "serwer zwrócił nieprawidłowy numer wersji".to_string())?;
    let mut current = version_parts(current)
        .ok_or_else(|| "program ma nieprawidłowy numer wersji".to_string())?;
    let length = remote.len().max(current.len());
    remote.resize(length, 0);
    current.resize(length, 0);
    Ok(remote > current)
}

fn validate_sha256(value: &str) -> Result<(), String> {
    if value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        Ok(())
    } else {
        Err("serwer zwrócił nieprawidłową sumę SHA-256".to_string())
    }
}

fn sha256_file(path: &Path) -> Result<String, String> {
    let mut command = Command::new("certutil.exe");
    command
        .args(["-hashfile"])
        .arg(path)
        .arg("SHA256")
        .creation_flags(CREATE_NO_WINDOW);
    let output = command
        .output()
        .map_err(|error| format!("nie można uruchomić kontroli SHA-256: {}", error))?;
    if !output.status.success() {
        return Err("systemowa kontrola SHA-256 zakończyła się błędem".to_string());
    }
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| line.split_whitespace().collect::<String>())
        .find(|line| line.len() == 64 && line.bytes().all(|byte| byte.is_ascii_hexdigit()))
        .map(|line| line.to_ascii_lowercase())
        .ok_or_else(|| "nie można odczytać wyniku kontroli SHA-256".to_string())
}

fn verify_sha256(path: &Path, expected: &str) -> Result<(), String> {
    validate_sha256(expected)?;
    let actual = sha256_file(path)?;
    if actual.eq_ignore_ascii_case(expected) {
        Ok(())
    } else {
        Err("suma pobranego pliku różni się od sumy opublikowanej na serwerze".to_string())
    }
}

fn check_and_download_update() -> Result<Option<(String, String, PathBuf)>, String> {
    let version = read_static_text("version.txt", "numeru wersji")?
        .trim()
        .to_string();
    if !version_is_newer(&version, APP_VERSION)? {
        return Ok(None);
    }
    let changelog = read_static_text("changelog.txt", "listy zmian")
        .unwrap_or_else(|_| "Brak opisu zmian.".to_string());
    let expected_sha256 = read_static_text("Majher.exe.sha256", "sumy aktualizacji")?
        .trim()
        .to_string();
    validate_sha256(&expected_sha256)?;
    let destination = executable_directory()?.join(format!(
        ".majher-update-{}.exe",
        std::process::id()
    ));
    let _ = fs::remove_file(&destination);
    download_static_file("Majher.exe", &destination)?;
    verify_sha256(&destination, &expected_sha256).map_err(|error| {
        let _ = fs::remove_file(&destination);
        format!("aktualizacja nie przeszła weryfikacji: {}", error)
    })?;
    Ok(Some((version, changelog, destination)))
}

fn launch_update_replacement(downloaded_file: &Path) -> Result<(), String> {
    let current_exe = std::env::current_exe()
        .map_err(|error| format!("nie można ustalić nazwy programu: {}", error))?;
    let script = r#"
$old = $env:MAJHER_OLD_EXE
$new = $env:MAJHER_NEW_EXE
$targetPid = [int]$env:MAJHER_TARGET_PID
$backup = "$old.bak"
Wait-Process -Id $targetPid -ErrorAction SilentlyContinue
$installed = $false
for ($attempt = 0; $attempt -lt 30; $attempt++) {
    try {
        Copy-Item -LiteralPath $old -Destination $backup -Force
        Move-Item -LiteralPath $new -Destination $old -Force
        $installed = $true
        break
    } catch {
        Start-Sleep -Milliseconds 500
    }
}
if (-not $installed) {
    "Aktualizacja nie powiodła się podczas podmiany pliku." | Set-Content -LiteralPath "$old.update-error.txt"
    exit 1
}
try {
    Start-Process -FilePath $old -WorkingDirectory (Split-Path -Parent $old) -ErrorAction Stop
} catch {
    try {
        if (Test-Path -LiteralPath $backup) {
            Copy-Item -LiteralPath $backup -Destination $old -Force
            Start-Process -FilePath $old -WorkingDirectory (Split-Path -Parent $old)
        }
    } catch {}
    "Aktualizacja nie powiodła się podczas uruchamiania programu." | Set-Content -LiteralPath "$old.update-error.txt"
    exit 1
}
for ($attempt = 0; $attempt -lt 40; $attempt++) {
    if (-not (Test-Path -LiteralPath $backup)) {
        break
    }
    try {
        Remove-Item -LiteralPath $backup -Force -ErrorAction Stop
        break
    } catch {
        Start-Sleep -Milliseconds 250
    }
}
try {
    if (Test-Path -LiteralPath $backup) {
        Remove-Item -LiteralPath $backup -Force -ErrorAction SilentlyContinue
    }
    if (Test-Path -LiteralPath "$old.update-error.txt") {
        Remove-Item -LiteralPath "$old.update-error.txt" -Force -ErrorAction SilentlyContinue
    }
} catch {}
exit 0
"#;
    let mut command = Command::new("powershell.exe");
    command
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-WindowStyle",
            "Hidden",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            script,
        ])
        .env("MAJHER_OLD_EXE", &current_exe)
        .env("MAJHER_NEW_EXE", downloaded_file)
        .env("MAJHER_TARGET_PID", std::process::id().to_string())
        .creation_flags(CREATE_NO_WINDOW);
    command
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("nie można uruchomić procesu podmiany: {}", error))
}

fn update_backup_path(executable: &Path) -> PathBuf {
    let mut name = executable.as_os_str().to_os_string();
    name.push(".bak");
    PathBuf::from(name)
}

fn remove_file_with_retries(path: &Path, attempts: usize, delay: Duration) -> bool {
    for attempt in 0..attempts.max(1) {
        match fs::remove_file(path) {
            Ok(()) => return true,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return true,
            Err(_) if attempt + 1 < attempts.max(1) => thread::sleep(delay),
            Err(_) => return false,
        }
    }
    false
}

fn cleanup_previous_update_backup() {
    let Ok(current_exe) = std::env::current_exe() else {
        return;
    };
    let backup = update_backup_path(&current_exe);
    if !backup.is_file() {
        return;
    }
    thread::spawn(move || {
        remove_file_with_retries(&backup, 40, Duration::from_millis(250));
    });
}

fn seconds_arg(value: f64) -> String {
    format!("{:.3}", value.max(0.0))
}

fn human_duration(seconds: f64) -> String {
    let total = seconds.round().max(0.0) as u64;
    let hours = total / 3600;
    let minutes = (total % 3600) / 60;
    let secs = total % 60;
    if hours > 0 {
        format!("{}:{:02}:{:02}", hours, minutes, secs)
    } else {
        format!("{}:{:02}", minutes, secs)
    }
}

fn number_after(line: &str, marker: &str) -> Option<f64> {
    let pos = line.find(marker)?;
    let tail = line[pos + marker.len()..].trim_start();
    let number: String = tail
        .chars()
        .take_while(|ch| ch.is_ascii_digit() || *ch == '.' || *ch == '-')
        .collect();
    number.parse::<f64>().ok()
}

fn short_process_error(name: &str, output: &Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let combined = if stderr.trim().is_empty() { stdout.trim() } else { stderr.trim() };
    let message: String = combined.chars().take(900).collect();
    if message.is_empty() {
        format!("{} zakończył pracę z kodem błędu", name)
    } else {
        format!("{}: {}", name, message)
    }
}

fn short_stderr_error(name: &str, stderr: &[u8]) -> String {
    let stderr = String::from_utf8_lossy(stderr);
    let message: String = stderr.trim().chars().rev().take(1400).collect::<String>().chars().rev().collect();
    if message.is_empty() {
        format!("{} zakończył pracę z kodem błędu", name)
    } else {
        format!("{}: {}", name, message)
    }
}

unsafe fn refresh_file_list(list: Hwnd, files: &[PathBuf]) {
    SendMessageW(list, LB_RESETCONTENT, 0, 0);
    for file in files {
        let text = wide(&file.to_string_lossy());
        SendMessageW(list, LB_ADDSTRING, 0, text.as_ptr() as Lparam);
    }
}

unsafe fn choose_audio_files(owner: Hwnd) -> Option<Vec<PathBuf>> {
    let mut buffer = vec![0u16; 65536];
    let filter = wide_multi(&[
        tr("Pliki audio", "Audio files", "Audiodateien", "Аудиофайлы"),
        "*.mp3;*.m4a;*.m4b;*.aac;*.wav;*.flac;*.ogg;*.opus;*.wma;*.aiff;*.aif;*.ac3;*.mka;*.webm;*.mp4",
        tr("Wszystkie pliki", "All files", "Alle Dateien", "Все файлы"),
        "*.*",
    ]);
    let title = wide(tr("Wybierz pliki audio", "Choose audio files", "Audiodateien wählen", "Выберите аудиофайлы"));
    let mut ofn = OpenFileNameW {
        l_struct_size: std::mem::size_of::<OpenFileNameW>() as Dword,
        hwnd_owner: owner,
        h_instance: 0,
        lpstr_filter: filter.as_ptr(),
        lpstr_custom_filter: null_mut(),
        n_max_cust_filter: 0,
        n_filter_index: 1,
        lpstr_file: buffer.as_mut_ptr(),
        n_max_file: buffer.len() as Dword,
        lpstr_file_title: null_mut(),
        n_max_file_title: 0,
        lpstr_initial_dir: null(),
        lpstr_title: title.as_ptr(),
        flags: OFN_EXPLORER | OFN_FILEMUSTEXIST | OFN_ALLOWMULTISELECT | OFN_HIDEREADONLY,
        n_file_offset: 0,
        n_file_extension: 0,
        lpstr_def_ext: null(),
        l_cust_data: 0,
        lpfn_hook: null_mut(),
        lp_template_name: null(),
        pv_reserved: null_mut(),
        dw_reserved: 0,
        flags_ex: 0,
    };

    if GetOpenFileNameW(&mut ofn) == 0 {
        return None;
    }
    parse_multi_select_buffer(&buffer)
}

fn parse_multi_select_buffer(buffer: &[u16]) -> Option<Vec<PathBuf>> {
    let mut parts = Vec::new();
    let mut start = 0usize;
    for (index, value) in buffer.iter().enumerate() {
        if *value == 0 {
            if index == start {
                break;
            }
            parts.push(OsString::from_wide(&buffer[start..index]));
            start = index + 1;
        }
    }

    if parts.is_empty() {
        None
    } else if parts.len() == 1 {
        Some(vec![PathBuf::from(&parts[0])])
    } else {
        let root = PathBuf::from(&parts[0]);
        Some(parts.into_iter().skip(1).map(|name| root.join(name)).collect())
    }
}

unsafe fn choose_folder(owner: Hwnd) -> Option<PathBuf> {
    let title = wide(tr("Wybierz folder wyjściowy", "Choose output folder", "Ausgabeordner wählen", "Выберите папку вывода"));
    let mut display = vec![0u16; 260];
    let mut info = BrowseInfoW {
        hwnd_owner: owner,
        pidl_root: null_mut(),
        psz_display_name: display.as_mut_ptr(),
        lpsz_title: title.as_ptr(),
        ul_flags: BIF_RETURNONLYFSDIRS | BIF_NEWDIALOGSTYLE | BIF_EDITBOX,
        lpfn: null_mut(),
        l_param: 0,
        i_image: 0,
    };
    let pidl = SHBrowseForFolderW(&mut info);
    if pidl.is_null() {
        return None;
    }
    let mut path = vec![0u16; 260];
    let ok = SHGetPathFromIDListW(pidl, path.as_mut_ptr()) != 0;
    CoTaskMemFree(pidl);
    if ok {
        Some(PathBuf::from(wide_to_string(&path)))
    } else {
        None
    }
}

unsafe fn append_log(log: Hwnd, text: &str) {
    let localized = translate_runtime(text);
    let mut value = String::with_capacity(localized.len() + 2);
    value.push_str(&localized);
    value.push_str("\r\n");
    let wide_text = wide(&value);
    let len = SendMessageW(log, WM_GETTEXTLENGTH, 0, 0);
    SendMessageW(log, EM_SETSEL, len as Wparam, len);
    SendMessageW(log, EM_REPLACESEL, 0, wide_text.as_ptr() as Lparam);
}

fn post_log(hwnd: Hwnd, text: &str) {
    if hwnd == 0 {
        return;
    }
    let boxed = Box::new(text.to_string());
    unsafe {
        PostMessageW(hwnd, WM_APP_LOG, 0, Box::into_raw(boxed) as Lparam);
    }
}

fn show_message_on_ui(hwnd: Hwnd, text: &str) {
    let boxed = Box::new(text.to_string());
    unsafe {
        SendMessageW(
            hwnd,
            WM_APP_NOTICE,
            0,
            Box::into_raw(boxed) as Lparam,
        );
    }
}

fn post_progress(hwnd: Hwnd, position: u32) {
    if hwnd == 0 {
        return;
    }
    unsafe {
        PostMessageW(hwnd, WM_APP_PROGRESS, position.min(1000) as Wparam, 0);
    }
}

fn progress_between(start: u32, end: u32, local_per_mille: u32) -> u32 {
    start + (((end - start) as u64 * local_per_mille.min(1000) as u64) / 1000) as u32
}

unsafe fn set_progress(progress_bar: Hwnd, position: u32) {
    SendMessageW(progress_bar, PBM_SETPOS, position.min(1000) as Wparam, 0);
    NotifyWinEvent(
        EVENT_OBJECT_VALUECHANGE,
        progress_bar,
        OBJID_CLIENT,
        CHILDID_SELF as i32,
    );
}

unsafe fn get_window_text(hwnd: Hwnd) -> String {
    let len = GetWindowTextLengthW(hwnd).max(0) as usize;
    let mut buffer = vec![0u16; len + 1];
    let copied = GetWindowTextW(hwnd, buffer.as_mut_ptr(), buffer.len() as i32);
    wide_to_string(&buffer[..copied.max(0) as usize])
}

unsafe fn set_window_text(hwnd: Hwnd, text: &str) {
    let value = wide(text);
    SetWindowTextW(hwnd, value.as_ptr());
}

unsafe fn open_support_url(hwnd: Hwnd, url: &str) {
    let operation = wide("open");
    let target = wide(url);
    let result = ShellExecuteW(
        hwnd,
        operation.as_ptr(),
        target.as_ptr(),
        null(),
        null(),
        SW_SHOW,
    );
    if result <= 32 {
        show_message(
            hwnd,
            tr(
                "Nie udało się otworzyć strony wsparcia.",
                "Could not open the support page.",
                "Die Unterstützungsseite konnte nicht geöffnet werden.",
                "Не удалось открыть страницу поддержки.",
            ),
        );
    }
}

unsafe fn show_message(hwnd: Hwnd, text: &str) {
    let localized = translate_runtime(text);
    let text = wide(&localized);
    let title = wide(&app_window_title());
    MessageBoxW(hwnd, text.as_ptr(), title.as_ptr(), 0);
}

unsafe fn ask_yes_no(hwnd: Hwnd, text: &str) -> bool {
    let localized = translate_runtime(text);
    let text = wide(&localized);
    let title = wide(&app_window_title());
    MessageBoxW(
        hwnd,
        text.as_ptr(),
        title.as_ptr(),
        MB_YESNO | MB_ICONINFORMATION,
    ) == IDYES
}

unsafe fn ask_detector_hardware(hwnd: Hwnd) -> Option<bool> {
    let text = wide(tr(
        "Wykryto kartę NVIDIA. Wybierz sposób pierwszego przygotowania wykrywania rozdziałów.\n\nTak: pobierz model, silnik CPU i przyspieszenie NVIDIA CUDA, łącznie około 840 MB. Jeżeli CUDA nie zadziała, Majher automatycznie przejdzie na CPU.\n\nNie: pobierz tylko model i silnik CPU, łącznie około 190 MB.\n\nAnuluj: nie rozpoczynaj zadania.\n\nPliki zostaną zapisane w tools\\chapter-detector obok programu.",
        "An NVIDIA card was detected. Choose how to prepare chapter detection.\n\nYes: download the model, CPU engine and NVIDIA CUDA acceleration, about 840 MB total. If CUDA fails, Majher will switch to CPU.\n\nNo: download only the model and CPU engine, about 190 MB.\n\nCancel: do not start.\n\nFiles will be saved in tools\\chapter-detector next to the program.",
        "Eine NVIDIA-Karte wurde erkannt. Wählen Sie die Vorbereitung der Kapitelerkennung.\n\nJa: Modell, CPU-Engine und NVIDIA-CUDA-Beschleunigung herunterladen, insgesamt ca. 840 MB. Falls CUDA nicht funktioniert, wechselt Majher zur CPU.\n\nNein: nur Modell und CPU-Engine herunterladen, ca. 190 MB.\n\nAbbrechen: Aufgabe nicht starten.\n\nDateien werden in tools\\chapter-detector neben dem Programm gespeichert.",
        "Обнаружена видеокарта NVIDIA. Выберите способ подготовки распознавания глав.\n\nДа: загрузить модель, движок CPU и ускорение NVIDIA CUDA, всего около 840 МБ. Если CUDA не заработает, Majher перейдет на CPU.\n\nНет: загрузить только модель и движок CPU, около 190 МБ.\n\nОтмена: не запускать задачу.\n\nФайлы будут сохранены в tools\\chapter-detector рядом с программой.",
    ));
    let title = wide(&app_window_title());
    match MessageBoxW(
        hwnd,
        text.as_ptr(),
        title.as_ptr(),
        MB_YESNOCANCEL | MB_ICONINFORMATION,
    ) {
        IDYES => Some(true),
        IDNO => Some(false),
        _ => None,
    }
}

unsafe fn show_success_and_offer_folders(hwnd: Hwnd, summary: &CompletionSummary) {
    let folder_question = if summary.output_directories.len() == 1 {
        tr(
            "Czy chcesz przejść do folderu z pociętą książką?",
            "Open the folder containing the split book?",
            "Den Ordner mit dem geteilten Buch öffnen?",
            "Открыть папку с разделенной книгой?",
        )
    } else {
        tr(
            "Czy chcesz otworzyć foldery z pociętymi książkami?",
            "Open the folders containing the split books?",
            "Die Ordner mit den geteilten Büchern öffnen?",
            "Открыть папки с разделенными книгами?",
        )
    };
    let message = format_tr(
        "Sukces.\n\nLiczba plików wejściowych: {}.\nLiczba utworzonych części: {}.\n\n{}",
        "Success.\n\nInput files: {}.\nCreated parts: {}.\n\n{}",
        "Erfolg.\n\nEingabedateien: {}.\nErstellte Teile: {}.\n\n{}",
        "Готово.\n\nИсходных файлов: {}.\nСоздано частей: {}.\n\n{}",
        &[
            summary.input_files_finished.to_string(),
            summary.output_files_created.to_string(),
            folder_question.to_string(),
        ],
    );
    let message = wide(&message);
    let title = wide(&app_window_title());
    let answer = MessageBoxW(
        hwnd,
        message.as_ptr(),
        title.as_ptr(),
        MB_YESNO | MB_ICONINFORMATION,
    );
    if answer != IDYES {
        return;
    }

    let operation = wide("open");
    let mut failed = 0usize;
    for directory in &summary.output_directories {
        let directory = wide(&directory.to_string_lossy());
        let result = ShellExecuteW(
            hwnd,
            operation.as_ptr(),
            directory.as_ptr(),
            null(),
            null(),
            SW_SHOW,
        );
        if result <= 32 {
            failed += 1;
        }
    }
    if failed > 0 {
        show_message(hwnd, tr("Nie udało się otworzyć jednego z folderów wynikowych.", "Could not open one of the output folders.", "Einer der Ausgabeordner konnte nicht geöffnet werden.", "Не удалось открыть одну из папок вывода."));
    }
}

fn same_path(a: &Path, b: &Path) -> bool {
    match (fs::canonicalize(a), fs::canonicalize(b)) {
        (Ok(a), Ok(b)) => a == b,
        _ => a == b,
    }
}

fn wide(text: &str) -> Vec<u16> {
    OsStr::new(text).encode_wide().chain(std::iter::once(0)).collect()
}

fn wide_multi(parts: &[&str]) -> Vec<u16> {
    let mut result = Vec::new();
    for part in parts {
        result.extend(OsStr::new(part).encode_wide());
        result.push(0);
    }
    result.push(0);
    result
}

fn wide_to_string(buffer: &[u16]) -> String {
    let end = buffer.iter().position(|value| *value == 0).unwrap_or(buffer.len());
    OsString::from_wide(&buffer[..end]).to_string_lossy().into_owned()
}

fn app_window_title() -> String {
    format!("{} {}", APP_TITLE, APP_VERSION)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ffmpeg_silence_numbers() {
        assert_eq!(number_after("[silencedetect] silence_start: 123.45", "silence_start:"), Some(123.45));
        assert_eq!(number_after("[silencedetect] silence_end: 130.2 | silence_duration: 6.75", "silence_end:"), Some(130.2));
    }

    #[test]
    fn builds_fixed_segments_without_silence() {
        let segments = build_segments(3700.0, 1800.0, &[]);
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0], (0.0, 1800.0));
        assert_eq!(segments[1], (1800.0, 3700.0));
    }

    #[test]
    fn prefers_nearby_silence_boundary() {
        let segments = build_segments(4000.0, 1800.0, &[1770.0, 3610.0]);
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0], (0.0, 1770.0));
        assert_eq!(segments[1], (1770.0, 4000.0));
    }

    #[test]
    fn maps_file_progress_into_whole_job() {
        assert_eq!(progress_between(250, 500, 0), 250);
        assert_eq!(progress_between(250, 500, 500), 375);
        assert_eq!(progress_between(250, 500, 1000), 500);
    }

    #[test]
    fn chooses_longest_duration_from_container_and_streams() {
        assert_eq!(parse_longest_duration("600.0\nN/A\n4800.25\n"), Some(4800.25));
    }

    #[test]
    fn splits_eighty_minutes_into_ten_minute_parts() {
        let segments = build_segments(80.0 * 60.0, 10.0 * 60.0, &[]);
        assert_eq!(segments.len(), 8);
        assert_eq!(segments.first(), Some(&(0.0, 600.0)));
        assert_eq!(segments.last(), Some(&(4200.0, 4800.0)));
    }

    #[test]
    fn parses_all_supported_language_codes() {
        assert_eq!(parse_language_setting("[Majher]\nlanguage=pl\n"), Some(Language::Polish));
        assert_eq!(parse_language_setting("language=en"), Some(Language::English));
        assert_eq!(parse_language_setting("language=de"), Some(Language::German));
        assert_eq!(parse_language_setting("language=ru"), Some(Language::Russian));
        assert_eq!(parse_language_setting("language=xx"), None);
        assert_eq!(parse_theme_setting("[Majher]\ntheme=system\n"), Some(ThemePreference::System));
        assert_eq!(parse_theme_setting("theme=light"), Some(ThemePreference::Light));
        assert_eq!(parse_theme_setting("theme=dark"), Some(ThemePreference::Dark));
        assert_eq!(parse_theme_setting("theme=unknown"), None);
        assert!(dark_theme_for(ThemePreference::System, true));
        assert!(!dark_theme_for(ThemePreference::System, false));
        assert!(!dark_theme_for(ThemePreference::Light, true));
        assert!(dark_theme_for(ThemePreference::Dark, false));
    }
    #[test]
    fn compares_update_versions_including_requested_1_01_format() {
        assert_eq!(version_is_newer("1.01", "1.0"), Ok(true));
        assert_eq!(version_is_newer("1.02", "1.01"), Ok(true));
        assert_eq!(version_is_newer("1.0", "1.0"), Ok(false));
        assert_eq!(version_is_newer("1.0.1", "1.0"), Ok(true));
    }

    #[test]
    fn includes_version_in_window_title() {
        assert_eq!(app_window_title(), "Majher 1.9");
    }

    #[test]
    fn parses_embedded_chapter_starts() {
        let chapters = parse_chapter_starts(
            "start_time=0.000000\nstart_time=615.250000\nstart_time=1210.500000\n",
        );
        assert_eq!(chapters, vec![615.25, 1210.5]);
    }

    #[test]
    fn recognizes_polish_audiobook_announcements() {
        assert_eq!(audiobook_marker_score("Rozdział piąty. Powrót."), 2);
        assert_eq!(audiobook_marker_score("Kaseta druga, strona B."), 2);
        assert_eq!(audiobook_marker_score("Część III"), 2);
        assert_eq!(audiobook_marker_score("Epilog"), 2);
        assert_eq!(audiobook_marker_score("To była część naszego planu."), 0);
    }

    #[test]
    fn keeps_detected_chapters_and_splits_only_long_ones() {
        let segments = build_audiobook_segments(
            100.0 * 60.0,
            30.0 * 60.0,
            &[20.0 * 60.0, 70.0 * 60.0],
            &[50.0 * 60.0],
        );
        assert_eq!(
            segments,
            vec![
                (0.0, 1200.0),
                (1200.0, 3000.0),
                (3000.0, 4200.0),
                (4200.0, 6000.0)
            ]
        );
    }

    #[test]
    fn groups_nearby_long_pauses_for_model_checks() {
        let points = [
            SilencePoint {
                start: 100.0,
                end: 102.0,
            },
            SilencePoint {
                start: 130.0,
                end: 134.0,
            },
            SilencePoint {
                start: 300.0,
                end: 303.0,
            },
        ];
        let selected = recognition_candidates(&points, 500.0);
        assert_eq!(selected.len(), 2);
        assert_eq!(selected[0], points[1]);
        assert_eq!(selected[1], points[2]);
    }

    #[test]
    fn builds_one_whisper_response_for_many_samples() {
        let jobs = vec![
            (
                PathBuf::from(r"C:\test z odstępem\okno-0001.wav"),
                PathBuf::from(r"C:\test z odstępem\tekst-0001"),
            ),
            (
                PathBuf::from(r"C:\test z odstępem\okno-0002.wav"),
                PathBuf::from(r"C:\test z odstępem\tekst-0002"),
            ),
        ];
        let response = whisper_batch_response(
            Path::new(r"C:\modele\ggml-small-q5_1.bin"),
            &jobs,
            ResourceProfile::Fast,
            true,
        );
        let arguments: Vec<&str> = response.lines().collect();
        assert_eq!(
            arguments
                .iter()
                .filter(|argument| **argument == "-m")
                .count(),
            1
        );
        assert_eq!(
            arguments
                .iter()
                .filter(|argument| **argument == "-f")
                .count(),
            2
        );
        assert_eq!(
            arguments
                .iter()
                .filter(|argument| **argument == "-of")
                .count(),
            2
        );
        assert!(arguments.contains(&"-fa"));
        assert!(arguments.contains(&r"C:\test z odstępem\okno-0001.wav"));
    }

    #[test]
    fn removes_stale_update_backup() {
        let executable = std::env::temp_dir().join(format!(
            "majher-backup-cleanup-test-{}.exe",
            std::process::id()
        ));
        let backup = update_backup_path(&executable);
        fs::write(&backup, b"old version").unwrap();
        assert!(remove_file_with_retries(
            &backup,
            1,
            Duration::from_millis(0)
        ));
        assert!(!backup.exists());
    }

    #[test]
    fn builds_static_update_file_url() {
        let url = static_url("version.txt").unwrap();
        assert!(url.starts_with(
            "https://github.com/proszczok2002/majher/releases/latest/download/version.txt?t="
        ));
        assert!(static_url("../sekret").is_err());
    }

    #[test]
    #[ignore = "test sieciowy uruchamiany ręcznie"]
    fn downloads_live_version_with_windows_urlmon() {
        let path = std::env::temp_dir().join(format!(
            "majher-live-version-test-{}.txt",
            std::process::id()
        ));
        let _ = fs::remove_file(&path);
        download_static_file("version.txt", &path).unwrap();
        let version = fs::read_to_string(&path).unwrap();
        let _ = fs::remove_file(&path);
        assert!(version_parts(version.trim()).is_some());
    }

    #[test]
    #[ignore = "test sieciowy uruchamiany ręcznie"]
    fn downloads_live_chapter_detector_config() {
        let config = read_static_text(CHAPTER_CONFIG_FILE, "konfiguracji modelu").unwrap();
        let parsed = parse_chapter_detector_config(&config).unwrap();
        assert_eq!(parsed.model_size, 190_085_487);
        assert!(parsed.model_url.ends_with(CHAPTER_MODEL_FILE));
    }

    #[test]
    #[ignore = "test sieciowy pobiera około 8 MB"]
    fn downloads_and_verifies_real_cpu_engine_package() {
        let config = read_static_text(CHAPTER_CONFIG_FILE, "konfiguracji modelu").unwrap();
        let parsed = parse_chapter_detector_config(&config).unwrap();
        let archive = std::env::temp_dir().join(format!(
            "majher-whisper-engine-test-{}.zip",
            std::process::id()
        ));
        let _ = fs::remove_file(&archive);
        download_url_cancellable(
            &parsed.cpu_url,
            &archive,
            &AtomicBool::new(false),
            0,
            "Test pobierania",
        )
        .unwrap();
        verify_sha256(&archive, &parsed.cpu_sha256).unwrap();
        let _ = fs::remove_file(&archive);
    }

    #[test]
    #[ignore = "test funkcjonalny wymaga pobranego silnika, modelu i próbki WAV"]
    fn transcribes_real_polish_chapter_announcement() {
        let root = PathBuf::from(
            std::env::var("MAJHER_DETECTOR_TEST_ROOT")
                .expect("ustaw MAJHER_DETECTOR_TEST_ROOT"),
        );
        let cpu_exe = find_file_recursive(&root.join("engine"), "whisper-cli.exe")
            .expect("brak whisper-cli.exe");
        let mut detector = ChapterDetector {
            cpu_exe: cpu_exe.clone(),
            primary_exe: cpu_exe,
            model: root.join(CHAPTER_MODEL_FILE),
            using_nvidia: false,
        };
        let prefix = root.join("rust-test-result");
        let text = transcribe_recognition_window(
            &mut detector,
            &root.join("audiobook-test.wav"),
            &prefix,
            ResourceProfile::Normal,
            &AtomicBool::new(false),
            0,
        )
        .unwrap();
        let _ = fs::remove_file(prefix.with_extension("txt"));
        assert!(audiobook_marker_score(&text) >= 2, "wynik modelu: {}", text);
    }

    #[test]
    #[ignore = "test funkcjonalny wymaga paczki CUDA, modelu i próbki WAV"]
    fn transcribes_many_samples_in_one_cuda_batch() {
        let root = PathBuf::from(
            std::env::var("MAJHER_CUDA_TEST_ROOT").expect("ustaw MAJHER_CUDA_TEST_ROOT"),
        );
        let cuda_exe =
            find_file_recursive(&root, "whisper-cli.exe").expect("brak whisper-cli.exe CUDA");
        let device = verify_nvidia_engine(&cuda_exe).unwrap();
        assert!(device.contains("NVIDIA"));
        let mut detector = ChapterDetector {
            cpu_exe: cuda_exe.clone(),
            primary_exe: cuda_exe,
            model: root.join(CHAPTER_MODEL_FILE),
            using_nvidia: true,
        };
        let jobs: Vec<(PathBuf, PathBuf)> = (0..3)
            .map(|index| {
                (
                    root.join("jfk.wav"),
                    root.join(format!("rust-cuda-batch-result-{}", index)),
                )
            })
            .collect();
        let results = transcribe_recognition_batch(
            &mut detector,
            &jobs,
            ResourceProfile::Fast,
            &AtomicBool::new(false),
            0,
            &root,
        )
        .unwrap();
        for (_, prefix) in &jobs {
            let _ = fs::remove_file(prefix.with_extension("txt"));
        }
        assert!(detector.using_nvidia);
        assert_eq!(results.len(), 3);
        assert!(results.iter().all(Result::is_ok));
    }

    #[test]
    #[ignore = "test funkcjonalny wymaga pobranego silnika, modelu, FFmpeg i próbki WAV"]
    fn detects_real_boundary_in_polish_audiobook_sample() {
        let root = PathBuf::from(
            std::env::var("MAJHER_DETECTOR_TEST_ROOT")
                .expect("ustaw MAJHER_DETECTOR_TEST_ROOT"),
        );
        let cpu_exe = find_file_recursive(&root.join("engine"), "whisper-cli.exe")
            .expect("brak whisper-cli.exe");
        let mut detector = ChapterDetector {
            cpu_exe: cpu_exe.clone(),
            primary_exe: cpu_exe,
            model: root.join(CHAPTER_MODEL_FILE),
            using_nvidia: false,
        };
        let sample = root.join("audiobook-test.wav");
        let duration = probe_duration(
            Path::new("ffprobe.exe"),
            &sample,
            ResourceProfile::Normal,
        )
        .unwrap();
        let cancel = AtomicBool::new(false);
        let (silences, _) = detect_silence_points(
            Path::new("ffmpeg.exe"),
            &sample,
            duration,
            ResourceProfile::Normal,
            &cancel,
            0,
            0,
            300,
        )
        .unwrap();
        assert!(
            silences
                .iter()
                .any(|point| (point.midpoint() - 10.4055).abs() < 0.5),
            "wykryte pauzy: {:?}",
            silences
        );
        let boundaries = recognize_audiobook_boundaries(
            &mut detector,
            Path::new("ffmpeg.exe"),
            &sample,
            duration,
            &silences,
            ResourceProfile::Normal,
            &cancel,
            0,
            0,
            1000,
        )
        .unwrap();
        assert_eq!(boundaries.len(), 1);
        assert!((boundaries[0] - 10.4055).abs() < 0.1);
    }

}
