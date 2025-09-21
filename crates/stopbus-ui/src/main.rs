use std::ffi::c_void;
use std::iter;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;

use stopbus_core::{DriveReport, GameEvent, GameState, MessageKind, HAND_SIZE};
use windows::core::{w, Error, Result, PCWSTR};
use windows::Win32::Foundation::{
    FreeLibrary, BOOL, COLORREF, HINSTANCE, HMODULE, HWND, LPARAM, LRESULT, RECT, WPARAM,
};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, BitBlt, CreateCompatibleDC, CreateSolidBrush, DeleteDC, DeleteObject, EndPaint,
    FillRect, InvalidateRect, LoadBitmapW, SelectObject, SetBkColor, SetTextColor, StretchBlt,
    TextOutW, HBITMAP, HDC, PAINTSTRUCT, SRCCOPY,
};
use windows::Win32::System::LibraryLoader::{
    GetModuleHandleW, LoadLibraryExW, LOAD_LIBRARY_AS_DATAFILE, LOAD_LIBRARY_AS_IMAGE_RESOURCE,
};
use windows::Win32::UI::Input::KeyboardAndMouse::EnableWindow;
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreateMenu, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyWindow,
    DispatchMessageW, GetMessageW, GetWindowLongPtrW, LoadCursorW, LoadIconW, MessageBoxW,
    PostQuitMessage, RegisterClassExW, SetMenu, SetWindowLongPtrW, ShowWindow, TranslateMessage,
    BS_DEFPUSHBUTTON, BS_PUSHBUTTON, CREATESTRUCTW, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT,
    GWLP_USERDATA, HMENU, MB_ICONEXCLAMATION, MB_ICONINFORMATION, MB_OK, MF_POPUP, MF_STRING, MSG,
    SW_SHOWDEFAULT, WINDOW_EX_STYLE, WINDOW_STYLE, WM_COMMAND, WM_CREATE, WM_DESTROY,
    WM_LBUTTONDOWN, WM_LBUTTONUP, WM_NCDESTROY, WM_PAINT, WNDCLASSEXW, WS_CAPTION, WS_CHILD,
    WS_MINIMIZEBOX, WS_OVERLAPPED, WS_SYSMENU, WS_VISIBLE,
};

const WINDOW_CLASS_NAME: PCWSTR = w!("StopBusMainWindow");
const WINDOW_TITLE: PCWSTR = w!("Stop the Bus (Rust modernization)");
const LEGACY_RES_PATH: &str = "STOPBUS.RES";

const WINDOW_WIDTH: i32 = 600;
const WINDOW_HEIGHT: i32 = 400;
const BACKGROUND_COLOR: u32 = 0x0000_8000;

const STACK_POSITION: (i32, i32) = (10, 40);
const TOP_CARD_POSITION: (i32, i32) = (110, 40);
const PLAYER1_CARD_POSITIONS: [(i32, i32); HAND_SIZE] = [(10, 200), (110, 200), (210, 200)];
const OPPONENT_CARD_POSITIONS: [[(i32, i32); HAND_SIZE]; 3] = [
    [(101, 6), (152, 6), (203, 6)],
    [(101, 63), (152, 63), (203, 63)],
    [(101, 120), (152, 120), (203, 120)],
];

const DECK_LABEL_POSITION: (i32, i32) = (20, 15);
const HAND_LABEL_POSITION: (i32, i32) = (20, 170);
const LIVES_HEADING_POSITION: (i32, i32) = (400, 55);
const SCORE_HEADING_POSITION: (i32, i32) = (480, 55);
const PLAYER_LABEL_X: i32 = 400;
const PLAYER_LABEL_BASE_Y: i32 = 75;
const PLAYER_LABEL_STEP: i32 = 15;
const POINTER_BASE_Y: i32 = 60;
const LIVES_VALUE_X: i32 = 470;
const SCORE_VALUE_X: i32 = 540;
const POINTER_X: i32 = 380;
const STICK_MARKER_X: i32 = 360;
const START_PLAYER_POSITION: (i32, i32) = (400, 150);

const BUTTON_WIDTH: i32 = 50;
const BUTTON_HEIGHT: i32 = 50;
const BUTTON_STICK_POS: (i32, i32) = (400, 200);
const BUTTON_DEAL_POS: (i32, i32) = (500, 200);
const BUTTON_EXIT_POS: (i32, i32) = (500, 260);
const BUTTON_OK_POS: (i32, i32) = (400, 260);

const CARD_WIDTH: i32 = 71;
const CARD_HEIGHT: i32 = 96;
const SMALL_CARD_WIDTH: i32 = 41;
const SMALL_CARD_HEIGHT: i32 = 55;

const CM_GAME_DEAL: usize = 100;
const CM_GAME_OPTIONS: usize = 101;
const CM_GAME_EXIT: usize = 102;
const CM_HELP_CONTENTS: usize = 900;
const CM_HELP_USING: usize = 901;
const CM_HELP_ABOUT: usize = 999;

const ID_STICK_BUTTON: usize = 1000;
const ID_DEAL_BUTTON: usize = 1001;
const ID_EXIT_BUTTON: usize = 1002;
const ID_OK_BUTTON: usize = 1003;

struct WindowState {
    game: GameState,
    legacy_module: Option<HMODULE>,
    cards: Vec<HBITMAP>,
    card_back: HBITMAP,
    card_cross: HBITMAP,
    stick_button: Option<HWND>,
    deal_button: Option<HWND>,
    exit_button: Option<HWND>,
    ok_button: Option<HWND>,
    awaiting_human: bool,
    pending_card: Option<usize>,
    stack_pressed: bool,
}

impl WindowState {
    fn new(instance: HINSTANCE) -> Result<Self> {
        let legacy_module = load_legacy_resource_module();
        let resource_instance = legacy_module
            .map(|module| HINSTANCE(module.0))
            .unwrap_or(instance);

        let mut cards = Vec::with_capacity(52);
        for id in 1..=52 {
            cards.push(unsafe { load_bitmap(resource_instance, id as u16)? });
        }

        let card_back = unsafe { load_bitmap(resource_instance, 53)? };
        let card_cross = unsafe { load_bitmap(resource_instance, 54)? };

        Ok(Self {
            game: GameState::default(),
            legacy_module,
            cards,
            card_back,
            card_cross,
            stick_button: None,
            deal_button: None,
            exit_button: None,
            ok_button: None,
            awaiting_human: false,
            pending_card: None,
            stack_pressed: false,
        })
    }

    fn process_report(&mut self, hwnd: HWND, report: DriveReport) {
        for event in &report.events {
            self.show_event(hwnd, event);
        }

        self.update_button_states(&report);

        unsafe {
            let _ = InvalidateRect(hwnd, None, BOOL(1));
        }
    }

    fn update_button_states(&mut self, report: &DriveReport) {
        let game_over = report.game_over();
        let human_alive = self.game.lives()[0] > 0;
        self.awaiting_human = report.awaiting_human && human_alive && !game_over;

        Self::set_button_enabled(&self.ok_button, self.awaiting_human);

        let stick_enabled = self.awaiting_human && self.game.human_can_stick();
        Self::set_button_enabled(&self.stick_button, stick_enabled);

        if !self.awaiting_human {
            self.pending_card = None;
            self.stack_pressed = false;
        }

        if game_over {
            Self::set_button_enabled(&self.ok_button, false);
            Self::set_button_enabled(&self.stick_button, false);
        }
    }

    fn show_event(&self, hwnd: HWND, event: &GameEvent) {
        let text = wide_string(&event.text);
        let title = wide_string("Stop the Bus");
        let icon = match event.kind {
            MessageKind::Info => MB_ICONINFORMATION,
            MessageKind::Alert => MB_ICONEXCLAMATION,
        };

        unsafe {
            MessageBoxW(
                hwnd,
                PCWSTR(text.as_ptr()),
                PCWSTR(title.as_ptr()),
                MB_OK | icon,
            );
        }
    }

    fn set_button_enabled(button: &Option<HWND>, enabled: bool) {
        if let Some(handle) = button {
            unsafe {
                let _ = EnableWindow(*handle, BOOL(enabled as i32));
            }
        }
    }

    fn paint(&mut self, hdc: HDC) {
        let mem_dc = unsafe { CreateCompatibleDC(hdc) };
        if mem_dc.0.is_null() {
            return;
        }

        unsafe {
            let brush = CreateSolidBrush(COLORREF(BACKGROUND_COLOR));
            let rect = RECT {
                left: 0,
                top: 0,
                right: WINDOW_WIDTH,
                bottom: WINDOW_HEIGHT,
            };
            FillRect(hdc, &rect, brush);
            let _ = DeleteObject(brush);
        }

        unsafe {
            draw_bitmap(
                hdc,
                mem_dc,
                self.card_back,
                STACK_POSITION.0,
                STACK_POSITION.1,
            );
        }

        for (slot, (x, y)) in PLAYER1_CARD_POSITIONS.iter().enumerate() {
            let bitmap = match self.game.hands[0][slot] {
                Some(card) => self.card_bitmap(card).unwrap_or(self.card_cross),
                None => self.card_cross,
            };

            unsafe {
                draw_bitmap(hdc, mem_dc, bitmap, *x, *y);
            }
        }

        for (player_index, positions) in OPPONENT_CARD_POSITIONS.iter().enumerate() {
            let opponent = player_index + 1;
            for (slot, (x, y)) in positions.iter().enumerate() {
                let bitmap = match self.game.hands[opponent][slot] {
                    Some(_) => self.card_back,
                    None => self.card_cross,
                };

                unsafe {
                    draw_small_bitmap(hdc, mem_dc, bitmap, *x, *y);
                }
            }
        }

        if let Some(card) = self.game.stack_top_card() {
            if let Some(bitmap) = self.card_bitmap(card) {
                unsafe {
                    draw_bitmap(
                        hdc,
                        mem_dc,
                        bitmap,
                        TOP_CARD_POSITION.0,
                        TOP_CARD_POSITION.1,
                    );
                }
            }
        }

        unsafe {
            let _ = SetBkColor(hdc, COLORREF(BACKGROUND_COLOR));
            let _ = SetTextColor(hdc, COLORREF(0x0000_0000));
        }

        draw_text(hdc, DECK_LABEL_POSITION.0, DECK_LABEL_POSITION.1, "Deck:");
        draw_text(
            hdc,
            HAND_LABEL_POSITION.0,
            HAND_LABEL_POSITION.1,
            "Your hand:",
        );
        draw_text(
            hdc,
            LIVES_HEADING_POSITION.0,
            LIVES_HEADING_POSITION.1,
            "Remaining Lives:",
        );
        draw_text(
            hdc,
            SCORE_HEADING_POSITION.0,
            SCORE_HEADING_POSITION.1,
            "Round scores:",
        );

        for (index, lives) in self.game.lives().iter().enumerate() {
            let label_y = PLAYER_LABEL_BASE_Y + PLAYER_LABEL_STEP * index as i32;
            let label = format!("Player {} -", index + 1);
            draw_text(hdc, PLAYER_LABEL_X, label_y, &label);

            let life_text = format!("{}", lives);
            draw_text(hdc, LIVES_VALUE_X, label_y, &life_text);

            let score_text = format!("{}", self.game.round_scores[index]);
            draw_text(hdc, SCORE_VALUE_X, label_y, &score_text);
        }

        let start_label = format!("Player {} to start", self.game.round_start_player() + 1);
        draw_text(
            hdc,
            START_PLAYER_POSITION.0,
            START_PLAYER_POSITION.1,
            &start_label,
        );

        let arrow_y = POINTER_BASE_Y + PLAYER_LABEL_STEP * (self.game.current_player() as i32 + 1);
        draw_text(hdc, POINTER_X, arrow_y, " ->");

        if let Some(stick_player) = self.game.stick_player() {
            let stick_y = POINTER_BASE_Y + PLAYER_LABEL_STEP * (stick_player as i32 + 1);
            draw_text(hdc, STICK_MARKER_X, stick_y, "***");
        }

        unsafe {
            let _ = DeleteDC(mem_dc);
        }
    }

    fn card_bitmap(&self, card: u8) -> Option<HBITMAP> {
        let index = card.checked_sub(1)? as usize;
        self.cards.get(index).copied()
    }

    fn hit_test_player_card(&self, x: i32, y: i32) -> Option<usize> {
        for (slot, (card_x, card_y)) in PLAYER1_CARD_POSITIONS.iter().enumerate() {
            if Self::point_in_rect(x, y, *card_x, *card_y, CARD_WIDTH, CARD_HEIGHT) {
                return Some(slot);
            }
        }
        None
    }

    fn stack_contains(&self, x: i32, y: i32) -> bool {
        Self::point_in_rect(
            x,
            y,
            STACK_POSITION.0,
            STACK_POSITION.1,
            CARD_WIDTH,
            CARD_HEIGHT,
        )
    }

    fn point_in_rect(x: i32, y: i32, left: i32, top: i32, width: i32, height: i32) -> bool {
        x >= left && x < left + width && y >= top && y < top + height
    }
}

impl Drop for WindowState {
    fn drop(&mut self) {
        unsafe {
            for bitmap in &self.cards {
                let _ = DeleteObject(*bitmap);
            }
            let _ = DeleteObject(self.card_back);
            let _ = DeleteObject(self.card_cross);
            if let Some(module) = self.legacy_module.take() {
                let _ = FreeLibrary(module);
            }
        }
    }
}

fn wide_path(path: &Path) -> Vec<u16> {
    path.as_os_str()
        .encode_wide()
        .chain(iter::once(0))
        .collect()
}

fn wide_string(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(iter::once(0)).collect()
}

fn make_int_resource(id: u16) -> PCWSTR {
    PCWSTR(id as usize as *const u16)
}

fn draw_text(hdc: HDC, x: i32, y: i32, text: &str) {
    let wide = wide_string(text);
    if wide.len() > 1 {
        let slice = &wide[..wide.len() - 1];
        let _ = unsafe { TextOutW(hdc, x, y, slice) };
    }
}

fn get_mouse_pos(lparam: LPARAM) -> (i32, i32) {
    let x = (lparam.0 & 0xFFFF) as u16 as i16 as i32;
    let y = ((lparam.0 >> 16) & 0xFFFF) as u16 as i16 as i32;
    (x, y)
}

unsafe fn create_button(
    parent: HWND,
    instance: HINSTANCE,
    text: &str,
    id: usize,
    position: (i32, i32),
    default: bool,
) -> Result<HWND> {
    let caption = wide_string(text);
    let style_bits = WS_CHILD.0
        | WS_VISIBLE.0
        | if default {
            BS_DEFPUSHBUTTON as u32
        } else {
            BS_PUSHBUTTON as u32
        };

    CreateWindowExW(
        WINDOW_EX_STYLE(0),
        w!("BUTTON"),
        PCWSTR(caption.as_ptr()),
        WINDOW_STYLE(style_bits),
        position.0,
        position.1,
        BUTTON_WIDTH,
        BUTTON_HEIGHT,
        parent,
        HMENU(id as isize as *mut c_void),
        instance,
        None,
    )
}

fn load_legacy_resource_module() -> Option<HMODULE> {
    let mut candidates = Vec::new();
    candidates.push(Path::new(LEGACY_RES_PATH).to_path_buf());
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(dir) = exe_path.parent() {
            candidates.push(dir.join(LEGACY_RES_PATH));
        }
    }

    for candidate in candidates {
        if !candidate.exists() {
            continue;
        }
        let wide = wide_path(&candidate);
        if let Ok(module) = unsafe {
            LoadLibraryExW(
                PCWSTR(wide.as_ptr()),
                None,
                LOAD_LIBRARY_AS_DATAFILE | LOAD_LIBRARY_AS_IMAGE_RESOURCE,
            )
        } {
            return Some(module);
        }
    }

    None
}

fn register_window_class(instance: HINSTANCE) -> Result<()> {
    let icon = unsafe {
        LoadIconW(
            None,
            windows::Win32::UI::WindowsAndMessaging::IDI_APPLICATION,
        )?
    };
    let cursor = unsafe { LoadCursorW(None, windows::Win32::UI::WindowsAndMessaging::IDC_ARROW)? };

    let background = unsafe { CreateSolidBrush(COLORREF(BACKGROUND_COLOR)) };

    let class = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(window_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: instance,
        hIcon: icon,
        hCursor: cursor,
        hbrBackground: background,
        lpszMenuName: PCWSTR::null(),
        lpszClassName: WINDOW_CLASS_NAME,
        hIconSm: icon,
    };

    match unsafe { RegisterClassExW(&class) } {
        0 => Err(Error::from_win32()),
        _ => Ok(()),
    }
}

fn create_main_window(instance: HINSTANCE, state: *mut WindowState) -> Result<HWND> {
    unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE(0),
            WINDOW_CLASS_NAME,
            WINDOW_TITLE,
            WINDOW_STYLE(WS_OVERLAPPED.0 | WS_CAPTION.0 | WS_SYSMENU.0 | WS_MINIMIZEBOX.0),
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
            None,
            HMENU::default(),
            instance,
            Some(state.cast::<c_void>()),
        )
    }
}

unsafe fn window_state_mut(hwnd: HWND) -> Option<&'static mut WindowState> {
    let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut WindowState;
    if ptr.is_null() {
        None
    } else {
        Some(&mut *ptr)
    }
}

unsafe fn draw_bitmap(hdc: HDC, mem_dc: HDC, bitmap: HBITMAP, x: i32, y: i32) {
    let previous = SelectObject(mem_dc, bitmap);
    if !previous.0.is_null() {
        let _ = BitBlt(hdc, x, y, CARD_WIDTH, CARD_HEIGHT, mem_dc, 0, 0, SRCCOPY);
        let _ = SelectObject(mem_dc, previous);
    }
}

unsafe fn draw_small_bitmap(hdc: HDC, mem_dc: HDC, bitmap: HBITMAP, x: i32, y: i32) {
    let previous = SelectObject(mem_dc, bitmap);
    if !previous.0.is_null() {
        let _ = StretchBlt(
            hdc,
            x,
            y,
            SMALL_CARD_WIDTH,
            SMALL_CARD_HEIGHT,
            mem_dc,
            0,
            0,
            CARD_WIDTH,
            CARD_HEIGHT,
            SRCCOPY,
        );
        let _ = SelectObject(mem_dc, previous);
    }
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_CREATE => {
            let create = &*(lparam.0 as *const CREATESTRUCTW);
            let state_ptr = create.lpCreateParams as *mut WindowState;

            let _ = create_menus(hwnd);
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, state_ptr as isize);

            if !state_ptr.is_null() {
                let state = &mut *state_ptr;
                let instance = create.hInstance;

                let stick = match create_button(
                    hwnd,
                    instance,
                    "Stick",
                    ID_STICK_BUTTON,
                    BUTTON_STICK_POS,
                    false,
                ) {
                    Ok(handle) => handle,
                    Err(_) => return DefWindowProcW(hwnd, message, wparam, lparam),
                };
                state.stick_button = Some(stick);

                let deal = match create_button(
                    hwnd,
                    instance,
                    "Deal",
                    ID_DEAL_BUTTON,
                    BUTTON_DEAL_POS,
                    false,
                ) {
                    Ok(handle) => handle,
                    Err(_) => return DefWindowProcW(hwnd, message, wparam, lparam),
                };
                state.deal_button = Some(deal);

                let exit = match create_button(
                    hwnd,
                    instance,
                    "Exit",
                    ID_EXIT_BUTTON,
                    BUTTON_EXIT_POS,
                    false,
                ) {
                    Ok(handle) => handle,
                    Err(_) => return DefWindowProcW(hwnd, message, wparam, lparam),
                };
                state.exit_button = Some(exit);

                let ok =
                    match create_button(hwnd, instance, "OK", ID_OK_BUTTON, BUTTON_OK_POS, true) {
                        Ok(handle) => handle,
                        Err(_) => return DefWindowProcW(hwnd, message, wparam, lparam),
                    };
                state.ok_button = Some(ok);

                let report = state.game.start_fresh();
                state.process_report(hwnd, report);
            }

            LRESULT(0)
        }
        WM_COMMAND => {
            let command = (wparam.0 & 0xFFFF) as usize;
            match command {
                CM_GAME_DEAL | ID_DEAL_BUTTON => {
                    if let Some(state) = window_state_mut(hwnd) {
                        let report = state.game.start_fresh();
                        state.process_report(hwnd, report);
                    }
                    LRESULT(0)
                }
                CM_GAME_EXIT | ID_EXIT_BUTTON => {
                    let _ = DestroyWindow(hwnd);
                    LRESULT(0)
                }
                ID_STICK_BUTTON => {
                    if let Some(state) = window_state_mut(hwnd) {
                        if let Some(report) = state.game.human_stick() {
                            state.process_report(hwnd, report);
                        }
                    }
                    LRESULT(0)
                }
                ID_OK_BUTTON => {
                    if let Some(state) = window_state_mut(hwnd) {
                        let report = state.game.advance_after_human_turn();
                        state.process_report(hwnd, report);
                    }
                    LRESULT(0)
                }
                CM_GAME_OPTIONS | CM_HELP_CONTENTS | CM_HELP_USING | CM_HELP_ABOUT => {
                    show_info(
                        hwnd,
                        match command {
                            CM_GAME_OPTIONS => "Options dialog not yet implemented.",
                            CM_HELP_CONTENTS => "Help contents will migrate from WinHelp soon.",
                            CM_HELP_USING => "Using Help placeholder.",
                            CM_HELP_ABOUT => "About dialog not yet ported.",
                            _ => "",
                        },
                    );
                    LRESULT(0)
                }
                _ => DefWindowProcW(hwnd, message, wparam, lparam),
            }
        }
        WM_LBUTTONDOWN => {
            let (x, y) = get_mouse_pos(lparam);
            if let Some(state) = window_state_mut(hwnd) {
                if state.awaiting_human && state.game.awaiting_human() {
                    state.pending_card = state.hit_test_player_card(x, y);
                    state.stack_pressed = state.stack_contains(x, y);
                } else {
                    state.pending_card = None;
                    state.stack_pressed = false;
                }
            }
            LRESULT(0)
        }
        WM_LBUTTONUP => {
            let (x, y) = get_mouse_pos(lparam);
            if let Some(state) = window_state_mut(hwnd) {
                let mut handled = false;

                if let Some(slot) = state.pending_card.take() {
                    if state.awaiting_human
                        && state.game.awaiting_human()
                        && state.hit_test_player_card(x, y) == Some(slot)
                    {
                        if let Some(report) = state.game.human_swap_with_stack(slot) {
                            state.process_report(hwnd, report);
                        }
                        handled = true;
                    }
                }

                if state.stack_pressed {
                    state.stack_pressed = false;
                    if state.awaiting_human
                        && state.game.awaiting_human()
                        && state.stack_contains(x, y)
                    {
                        if let Some(report) = state.game.human_draw_next_card() {
                            state.process_report(hwnd, report);
                        }
                        handled = true;
                    }
                }

                if handled {
                    return LRESULT(0);
                }
            }
            LRESULT(0)
        }
        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();
            let hdc = BeginPaint(hwnd, &mut ps);
            if let Some(state) = window_state_mut(hwnd) {
                state.game.update_round_scores();
                state.paint(hdc);
            }
            let _ = EndPaint(hwnd, &ps);
            LRESULT(0)
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        WM_NCDESTROY => {
            let state_ptr = SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0) as *mut WindowState;
            if !state_ptr.is_null() {
                drop(Box::from_raw(state_ptr));
            }
            DefWindowProcW(hwnd, message, wparam, lparam)
        }
        _ => DefWindowProcW(hwnd, message, wparam, lparam),
    }
}

unsafe fn load_bitmap(instance: HINSTANCE, id: u16) -> Result<HBITMAP> {
    let handle = LoadBitmapW(instance, make_int_resource(id));
    if handle.0.is_null() {
        Err(Error::from_win32())
    } else {
        Ok(handle)
    }
}

fn show_info(hwnd: HWND, message: &str) {
    let text = wide_string(message);
    let title = wide_string("Stop the Bus");
    let _ = unsafe {
        MessageBoxW(
            hwnd,
            PCWSTR(text.as_ptr()),
            PCWSTR(title.as_ptr()),
            MB_OK | MB_ICONINFORMATION,
        )
    };
}

unsafe fn create_menus(hwnd: HWND) -> Result<()> {
    let main_menu = CreateMenu()?;
    let game_menu = CreatePopupMenu()?;
    let help_menu = CreatePopupMenu()?;

    AppendMenuW(game_menu, MF_STRING, CM_GAME_DEAL, w!("&Deal"))?;
    AppendMenuW(game_menu, MF_STRING, CM_GAME_OPTIONS, w!("&Options"))?;
    AppendMenuW(game_menu, MF_STRING, CM_GAME_EXIT, w!("E&xit"))?;

    AppendMenuW(help_menu, MF_STRING, CM_HELP_CONTENTS, w!("&Contents"))?;
    AppendMenuW(help_menu, MF_STRING, CM_HELP_USING, w!("Using &Help"))?;
    AppendMenuW(help_menu, MF_STRING, CM_HELP_ABOUT, w!("&About"))?;

    AppendMenuW(
        main_menu,
        MF_POPUP | MF_STRING,
        game_menu.0 as usize,
        w!("&Game"),
    )?;
    AppendMenuW(
        main_menu,
        MF_POPUP | MF_STRING,
        help_menu.0 as usize,
        w!("&Help"),
    )?;

    SetMenu(hwnd, main_menu)?;
    Ok(())
}

fn main() -> Result<()> {
    unsafe {
        let module = GetModuleHandleW(None)?;
        let instance = HINSTANCE(module.0);
        register_window_class(instance)?;

        let state = Box::new(WindowState::new(instance)?);
        let state_ptr = Box::into_raw(state);
        let hwnd = match create_main_window(instance, state_ptr) {
            Ok(window) => window,
            Err(err) => {
                drop(Box::from_raw(state_ptr));
                return Err(err);
            }
        };

        let _ = ShowWindow(hwnd, SW_SHOWDEFAULT);

        let mut message = MSG::default();
        while GetMessageW(&mut message, HWND::default(), 0, 0).into() {
            let _ = TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    }

    Ok(())
}
