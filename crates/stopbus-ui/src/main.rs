use std::ffi::c_void;

use std::iter;

use std::os::windows::ffi::OsStrExt;

use std::path::Path;

use std::mem::size_of;

use std::ptr::copy_nonoverlapping;

use std::slice;

use stopbus_core::{DriveReport, GameEvent, GameState, MessageKind, DECK_SIZE, HAND_SIZE};

use windows::core::{w, Error, Result, PCWSTR};

use windows::Win32::Foundation::{
    FreeLibrary, BOOL, COLORREF, ERROR_SUCCESS, HINSTANCE, HMODULE, HWND, LPARAM, LRESULT, RECT,
    WPARAM,
};

use windows::Win32::Graphics::Gdi::{
    BeginPaint, BitBlt, CreateCompatibleDC, CreateDIBSection, CreateSolidBrush, DeleteDC,
    DeleteObject, EndPaint, FillRect, GetDC, InvalidateRect, LoadBitmapW, ReleaseDC, SelectObject,
    SetBkColor, SetPixelV, SetTextColor, StretchBlt, TextOutW, BITMAPINFO, BITMAPINFOHEADER,
    BI_RGB, DIB_RGB_COLORS, HBITMAP, HBRUSH, HDC, PAINTSTRUCT, RGBQUAD, SRCCOPY,
};

use windows::Win32::System::LibraryLoader::{
    FindResourceW, GetModuleHandleW, LoadLibraryExW, LoadResource, LockResource, SizeofResource,
    LOAD_LIBRARY_AS_DATAFILE, LOAD_LIBRARY_AS_IMAGE_RESOURCE,
};

use windows::Win32::System::Registry::{
    RegGetValueW, RegSetKeyValueW, HKEY_CURRENT_USER, REG_DWORD, RRF_RT_REG_DWORD,
};

use windows::Win32::UI::Input::KeyboardAndMouse::EnableWindow;

use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreateMenu, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyWindow,
    DialogBoxParamW, DispatchMessageW, EndDialog, GetClientRect, GetMessageW, GetWindowLongPtrW,
    GetWindowRect, IsWindowVisible, LoadCursorW, LoadIconW, MessageBoxW, PostMessageW,
    PostQuitMessage, RegisterClassExW, SendDlgItemMessageW, SetDlgItemTextW, SetMenu,
    SetWindowLongPtrW, ShowWindow, TranslateMessage, BM_GETCHECK, BM_SETCHECK, BS_DEFPUSHBUTTON,
    BS_PUSHBUTTON, CREATESTRUCTW, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, GWLP_USERDATA, HICON,
    HMENU, IDCANCEL, IDOK, MB_ICONEXCLAMATION, MB_ICONINFORMATION, MB_OK, MF_POPUP, MF_STRING, MSG,
    RT_BITMAP, SW_SHOWDEFAULT, WINDOW_EX_STYLE, WINDOW_STYLE, WM_COMMAND, WM_CREATE, WM_DESTROY,
    WM_INITDIALOG, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOVE, WM_NCDESTROY, WM_PAINT, WNDCLASSEXW,
    WNDPROC, WS_CAPTION, WS_CHILD, WS_EX_TOOLWINDOW, WS_MINIMIZEBOX, WS_OVERLAPPED, WS_POPUP,
    WS_SYSMENU, WS_VISIBLE,
};

const WINDOW_CLASS_NAME: PCWSTR = w!("StopBusMainWindow");

const WINDOW_TITLE: PCWSTR = w!("Stop the Bus (Rust modernization)");

const WM_APP_START: u32 = windows::Win32::UI::WindowsAndMessaging::WM_APP + 1;

const LEGACY_RES_PATH: &str = "STOPBUS.RES";

const APP_ICON_ID: u16 = 102;

const OPTIONS_DIALOG_NAME: PCWSTR = w!("OPTIONS");

const ABOUT_DIALOG_NAME: PCWSTR = w!("ABOUTBOX");

const ID_OPT_CARDS: i32 = 500;

const ID_OPT_STACK: i32 = 501;

const ID_OPT_SCORES: i32 = 502;

const ID_OPT_SAVE_EXIT: i32 = 503;

const ID_ABOUT_TITLE: i32 = 550;

const ID_ABOUT_COPYRIGHT: i32 = 551;

const ID_ABOUT_LICENSE_NAME: i32 = 552;

const ID_ABOUT_LICENSE_COMPANY: i32 = 553;

const ID_ABOUT_VERSION: i32 = 554;

const ID_ABOUT_RELEASE: i32 = 555;

const ID_ABOUT_ADDRESS: i32 = 556;

const LICENSE_NAME: &str = "Rust modernization build.";

const LICENSE_COMPANY: &str = "Maintained by the community.";

const LICENSE_ADDRESS: &str = "";

const BST_CHECKED: u32 = 1;

const BST_UNCHECKED: u32 = 0;

const CHEAT_CARDS_CLASS: PCWSTR = w!("StopBusCheatCards");

const CHEAT_STACK_CLASS: PCWSTR = w!("StopBusCheatStack");

const CHEAT_SCORES_CLASS: PCWSTR = w!("StopBusCheatScores");

const CHEAT_CARDS_SIZE: (i32, i32) = (320, 240);

const CHEAT_STACK_SIZE: (i32, i32) = (260, 180);

const CHEAT_SCORES_SIZE: (i32, i32) = (220, 160);

const CHEAT_WINDOW_PADDING: i32 = 20;

const CHEAT_STACK_PREVIEW: usize = 4;

const REGISTRY_SUBKEY: PCWSTR = w!(r"Software\StopBus\Modernization");

const REG_VALUE_CHEAT_CARDS_VISIBLE: PCWSTR = w!("CheatCardsVisible");

const REG_VALUE_CHEAT_STACK_VISIBLE: PCWSTR = w!("CheatStackVisible");

const REG_VALUE_CHEAT_SCORES_VISIBLE: PCWSTR = w!("CheatScoresVisible");

const REG_VALUE_CHEAT_CARDS_POS_X: PCWSTR = w!("CheatCardsPosX");

const REG_VALUE_CHEAT_CARDS_POS_Y: PCWSTR = w!("CheatCardsPosY");

const REG_VALUE_CHEAT_STACK_POS_X: PCWSTR = w!("CheatStackPosX");

const REG_VALUE_CHEAT_STACK_POS_Y: PCWSTR = w!("CheatStackPosY");

const REG_VALUE_CHEAT_SCORES_POS_X: PCWSTR = w!("CheatScoresPosX");

const REG_VALUE_CHEAT_SCORES_POS_Y: PCWSTR = w!("CheatScoresPosY");

const REG_VALUE_MAIN_WINDOW_POS_X: PCWSTR = w!("MainWindowPosX");

const REG_VALUE_MAIN_WINDOW_POS_Y: PCWSTR = w!("MainWindowPosY");

const REG_VALUE_SAVE_ON_EXIT: PCWSTR = w!("SaveOnExit");

const WINDOW_WIDTH: i32 = 600;

const WINDOW_HEIGHT: i32 = 400;

const BACKGROUND_COLOR: u32 = 0x0000_8000;

const STACK_POSITION: (i32, i32) = (10, 40);

const TOP_CARD_POSITION: (i32, i32) = (110, 40);

const PLAYER1_CARD_POSITIONS: [(i32, i32); HAND_SIZE] = [(10, 200), (110, 200), (210, 200)];

const DECK_LABEL_POSITION: (i32, i32) = (20, 15);

const HAND_LABEL_POSITION: (i32, i32) = (20, 170);

const LIVES_HEADING_POSITION: (i32, i32) = (400, 55);

const PLAYER_LABEL_X: i32 = 400;

const PLAYER_LABEL_BASE_Y: i32 = 75;

const PLAYER_LABEL_STEP: i32 = 15;

const POINTER_BASE_Y: i32 = 60;

const LIVES_VALUE_X: i32 = 470;

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

    module_instance: HINSTANCE,

    main_hwnd: Option<HWND>,

    main_window_pos: Option<(i32, i32)>,

    cards: Vec<HBITMAP>,

    card_back: HBITMAP,

    card_cross: HBITMAP,

    options_show_cheat_cards: bool,

    options_show_cheat_stack: bool,

    options_show_cheat_scores: bool,

    options_save_on_exit: bool,

    cheat_cards_window: Option<HWND>,

    cheat_stack_window: Option<HWND>,

    cheat_scores_window: Option<HWND>,

    cheat_cards_pos: (i32, i32),

    cheat_stack_pos: (i32, i32),

    cheat_scores_pos: (i32, i32),

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

        let cheat_base_y = WINDOW_HEIGHT + CHEAT_WINDOW_PADDING;

        let cheat_stack_x = CHEAT_CARDS_SIZE.0 + CHEAT_WINDOW_PADDING;

        let cheat_scores_x = cheat_stack_x + CHEAT_STACK_SIZE.0 + CHEAT_WINDOW_PADDING;

        let mut state = Self {
            game: GameState::default(),

            legacy_module,

            module_instance: instance,

            main_hwnd: None,

            main_window_pos: None,

            cards,

            card_back,

            card_cross,

            options_show_cheat_cards: false,

            options_show_cheat_stack: false,

            options_show_cheat_scores: false,

            options_save_on_exit: false,

            cheat_cards_window: None,

            cheat_stack_window: None,

            cheat_scores_window: None,

            cheat_cards_pos: (0, cheat_base_y),

            cheat_stack_pos: (cheat_stack_x, cheat_base_y),

            cheat_scores_pos: (cheat_scores_x, cheat_base_y),

            stick_button: None,

            deal_button: None,

            exit_button: None,

            ok_button: None,

            awaiting_human: false,

            pending_card: None,

            stack_pressed: false,
        };

        state.load_persisted_cheat_settings();

        state.show_or_hide_cheat_windows()?;

        state.update_cheat_windows();

        Ok(state)
    }

    fn process_report(&mut self, hwnd: HWND, report: DriveReport) {
        for event in &report.events {
            self.show_event(hwnd, event);
        }

        self.update_button_states(&report);

        unsafe {
            let _ = InvalidateRect(hwnd, None, BOOL(1));
        }

        self.update_cheat_windows();
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

        for (index, lives) in self.game.lives().iter().enumerate() {
            let label_y = PLAYER_LABEL_BASE_Y + PLAYER_LABEL_STEP * index as i32;

            let label = format!("Player {} -", index + 1);

            draw_text(hdc, PLAYER_LABEL_X, label_y, &label);

            let life_text = format!("{}", lives);

            draw_text(hdc, LIVES_VALUE_X, label_y, &life_text);
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

    fn show_or_hide_cheat_windows(&mut self) -> Result<()> {
        let ready = self.main_hwnd.is_some();

        if self.options_show_cheat_cards {
            if self.cheat_cards_window.is_none() && ready {
                self.cheat_cards_window = Some(create_cheat_cards_window(self)?);
            }
        } else if let Some(hwnd) = self.cheat_cards_window.take() {
            if let Some(pos) = unsafe { window_screen_position(hwnd) } {
                self.cheat_cards_pos = pos;
            }
            unsafe {
                let _ = DestroyWindow(hwnd);
            }
        }

        if self.options_show_cheat_stack {
            if self.cheat_stack_window.is_none() && ready {
                self.cheat_stack_window = Some(create_cheat_stack_window(self)?);
            }
        } else if let Some(hwnd) = self.cheat_stack_window.take() {
            if let Some(pos) = unsafe { window_screen_position(hwnd) } {
                self.cheat_stack_pos = pos;
            }
            unsafe {
                let _ = DestroyWindow(hwnd);
            }
        }

        if self.options_show_cheat_scores {
            if self.cheat_scores_window.is_none() && ready {
                self.cheat_scores_window = Some(create_cheat_scores_window(self)?);
            }
        } else if let Some(hwnd) = self.cheat_scores_window.take() {
            if let Some(pos) = unsafe { window_screen_position(hwnd) } {
                self.cheat_scores_pos = pos;
            }
            unsafe {
                let _ = DestroyWindow(hwnd);
            }
        }

        if ready {
            self.update_cheat_windows();
            self.capture_window_positions();
            self.persist_cheat_settings();
        }

        Ok(())
    }
    fn update_cheat_windows(&self) {
        unsafe {
            if let Some(hwnd) = self.cheat_cards_window {
                repaint_cheat_window(self, hwnd, paint_cheat_cards);
            }

            if let Some(hwnd) = self.cheat_stack_window {
                repaint_cheat_window(self, hwnd, paint_cheat_stack);
            }

            if let Some(hwnd) = self.cheat_scores_window {
                repaint_cheat_window(self, hwnd, paint_cheat_scores);
            }
        }
    }

    fn load_persisted_cheat_settings(&mut self) {
        if let Some(value) = registry_read_bool(REG_VALUE_CHEAT_CARDS_VISIBLE) {
            self.options_show_cheat_cards = value;
        }

        if let Some(value) = registry_read_bool(REG_VALUE_CHEAT_STACK_VISIBLE) {
            self.options_show_cheat_stack = value;
        }

        if let Some(value) = registry_read_bool(REG_VALUE_CHEAT_SCORES_VISIBLE) {
            self.options_show_cheat_scores = value;
        }

        if let Some(value) = registry_read_bool(REG_VALUE_SAVE_ON_EXIT) {
            self.options_save_on_exit = value;
        }

        if let Some(pos) =
            registry_read_point(REG_VALUE_MAIN_WINDOW_POS_X, REG_VALUE_MAIN_WINDOW_POS_Y)
        {
            self.main_window_pos = Some(pos);
        }

        if let Some(pos) =
            registry_read_point(REG_VALUE_CHEAT_CARDS_POS_X, REG_VALUE_CHEAT_CARDS_POS_Y)
        {
            self.cheat_cards_pos = pos;
        }

        if let Some(pos) =
            registry_read_point(REG_VALUE_CHEAT_STACK_POS_X, REG_VALUE_CHEAT_STACK_POS_Y)
        {
            self.cheat_stack_pos = pos;
        }

        if let Some(pos) =
            registry_read_point(REG_VALUE_CHEAT_SCORES_POS_X, REG_VALUE_CHEAT_SCORES_POS_Y)
        {
            self.cheat_scores_pos = pos;
        }
    }

    fn capture_window_positions(&mut self) {
        unsafe {
            if let Some(hwnd) = self.cheat_cards_window {
                if let Some(pos) = window_screen_position(hwnd) {
                    self.cheat_cards_pos = pos;
                }
            }
            if let Some(hwnd) = self.cheat_stack_window {
                if let Some(pos) = window_screen_position(hwnd) {
                    self.cheat_stack_pos = pos;
                }
            }
            if let Some(hwnd) = self.cheat_scores_window {
                if let Some(pos) = window_screen_position(hwnd) {
                    self.cheat_scores_pos = pos;
                }
            }
        }
    }

    fn persist_cheat_settings(&self) {
        registry_write_bool(REG_VALUE_SAVE_ON_EXIT, self.options_save_on_exit);

        if !self.options_save_on_exit {
            return;
        }

        registry_write_bool(REG_VALUE_CHEAT_CARDS_VISIBLE, self.options_show_cheat_cards);

        registry_write_bool(REG_VALUE_CHEAT_STACK_VISIBLE, self.options_show_cheat_stack);

        registry_write_bool(
            REG_VALUE_CHEAT_SCORES_VISIBLE,
            self.options_show_cheat_scores,
        );

        if let Some(pos) = self.main_window_pos {
            registry_write_point(
                REG_VALUE_MAIN_WINDOW_POS_X,
                REG_VALUE_MAIN_WINDOW_POS_Y,
                pos,
            );
        }

        registry_write_point(
            REG_VALUE_CHEAT_CARDS_POS_X,
            REG_VALUE_CHEAT_CARDS_POS_Y,
            self.cheat_cards_pos,
        );

        registry_write_point(
            REG_VALUE_CHEAT_STACK_POS_X,
            REG_VALUE_CHEAT_STACK_POS_Y,
            self.cheat_stack_pos,
        );

        registry_write_point(
            REG_VALUE_CHEAT_SCORES_POS_X,
            REG_VALUE_CHEAT_SCORES_POS_Y,
            self.cheat_scores_pos,
        );
    }

    fn show_options_dialog(&mut self, hwnd: HWND) -> Result<()> {
        let result = unsafe {
            DialogBoxParamW(
                self.module_instance,
                OPTIONS_DIALOG_NAME,
                hwnd,
                Some(options_dialog_proc),
                LPARAM(self as *mut _ as isize),
            )
        };

        let status = dialog_result(result);

        if status.is_ok() {
            self.show_or_hide_cheat_windows()?;

            self.update_cheat_windows();
        }

        status
    }

    fn show_about_dialog(&self, hwnd: HWND) -> Result<()> {
        let result = unsafe {
            DialogBoxParamW(
                self.module_instance,
                ABOUT_DIALOG_NAME,
                hwnd,
                Some(about_dialog_proc),
                LPARAM(self as *const _ as isize),
            )
        };

        dialog_result(result)
    }
}

impl Drop for WindowState {
    fn drop(&mut self) {
        self.capture_window_positions();
        self.persist_cheat_settings();

        unsafe {
            for bitmap in &self.cards {
                let _ = DeleteObject(*bitmap);
            }

            let _ = DeleteObject(self.card_back);

            let _ = DeleteObject(self.card_cross);

            if let Some(hwnd) = self.cheat_cards_window.take() {
                let _ = DestroyWindow(hwnd);
            }

            if let Some(hwnd) = self.cheat_stack_window.take() {
                let _ = DestroyWindow(hwnd);
            }

            if let Some(hwnd) = self.cheat_scores_window.take() {
                let _ = DestroyWindow(hwnd);
            }

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

unsafe fn repaint_cheat_window(
    state: &WindowState,

    hwnd: HWND,

    paint: unsafe fn(&WindowState, HWND, HDC),
) {
    let hdc = GetDC(hwnd);

    if hdc.0.is_null() {
        let _ = InvalidateRect(hwnd, None, BOOL(1));

        return;
    }

    paint(state, hwnd, hdc);

    let _ = ReleaseDC(hwnd, hdc);
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
    let icon = match unsafe { LoadIconW(instance, make_int_resource(APP_ICON_ID)) } {
        Ok(icon) => icon,

        Err(_) => unsafe {
            LoadIconW(
                None,
                windows::Win32::UI::WindowsAndMessaging::IDI_APPLICATION,
            )?
        },
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

        _ => {
            register_cheat_window_classes(instance, icon)?;

            Ok(())
        }
    }
}

fn create_main_window(instance: HINSTANCE, state: *mut WindowState) -> Result<HWND> {
    unsafe {
        let (x, y) = if state.is_null() {
            (CW_USEDEFAULT, CW_USEDEFAULT)
        } else {
            let state_ref = &mut *state;

            state_ref
                .main_window_pos
                .unwrap_or((CW_USEDEFAULT, CW_USEDEFAULT))
        };

        CreateWindowExW(
            WINDOW_EX_STYLE(0),
            WINDOW_CLASS_NAME,
            WINDOW_TITLE,
            WINDOW_STYLE(WS_OVERLAPPED.0 | WS_CAPTION.0 | WS_SYSMENU.0 | WS_MINIMIZEBOX.0),
            x,
            y,
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

unsafe fn soften_card_corners(hdc: HDC, x: i32, y: i32, width: i32, height: i32) {
    if width <= 0 || height <= 0 {
        return;
    }

    let color = COLORREF(BACKGROUND_COLOR);
    let right = x + width - 1;
    let bottom = y + height - 1;

    let _ = SetPixelV(hdc, x, y, color);
    if width > 1 {
        let _ = SetPixelV(hdc, x + 1, y, color);
        let _ = SetPixelV(hdc, right - 1, y, color);
    }
    if height > 1 {
        let _ = SetPixelV(hdc, x, y + 1, color);
        let _ = SetPixelV(hdc, x, bottom - 1, color);
    }

    let _ = SetPixelV(hdc, right, y, color);
    let _ = SetPixelV(hdc, x, bottom, color);
    let _ = SetPixelV(hdc, right, bottom, color);

    if height > 1 {
        let _ = SetPixelV(hdc, right, y + 1, color);
        let _ = SetPixelV(hdc, right, bottom - 1, color);
    }

    if width > 1 {
        let _ = SetPixelV(hdc, x + 1, bottom, color);
        let _ = SetPixelV(hdc, right - 1, bottom, color);
    }
}

unsafe fn draw_bitmap(hdc: HDC, mem_dc: HDC, bitmap: HBITMAP, x: i32, y: i32) {
    let previous = SelectObject(mem_dc, bitmap);

    if !previous.0.is_null() {
        let _ = BitBlt(hdc, x, y, CARD_WIDTH, CARD_HEIGHT, mem_dc, 0, 0, SRCCOPY);

        soften_card_corners(hdc, x, y, CARD_WIDTH, CARD_HEIGHT);

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

        soften_card_corners(hdc, x, y, SMALL_CARD_WIDTH, SMALL_CARD_HEIGHT);

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

                state.main_hwnd = Some(hwnd);

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

                state.main_hwnd = Some(hwnd);

                if let Some(pos) = unsafe { window_screen_position(hwnd) } {
                    state.main_window_pos = Some(pos);
                }

                if let Err(err) = state.show_or_hide_cheat_windows() {
                    let message = format!("Failed to restore cheat windows: {}", err);
                    show_error(hwnd, &message);
                }

                let _ = PostMessageW(hwnd, WM_APP_START, WPARAM(0), LPARAM(0));
            }

            LRESULT(0)
        }

        WM_MOVE => {
            if let Some(state) = window_state_mut(hwnd) {
                if let Some(pos) = unsafe { window_screen_position(hwnd) } {
                    state.main_window_pos = Some(pos);

                    state.persist_cheat_settings();
                }
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

                CM_GAME_OPTIONS => {
                    if let Some(state) = window_state_mut(hwnd) {
                        if let Err(err) = state.show_options_dialog(hwnd) {
                            show_error(hwnd, &format!("Failed to open Options dialog: {}", err));
                        }
                    }

                    LRESULT(0)
                }

                CM_HELP_ABOUT => {
                    if let Some(state) = window_state_mut(hwnd) {
                        if let Err(err) = state.show_about_dialog(hwnd) {
                            show_error(hwnd, &format!("Failed to open About dialog: {}", err));
                        }
                    }

                    LRESULT(0)
                }

                CM_HELP_CONTENTS | CM_HELP_USING => {
                    show_info(
                        hwnd,
                        match command {
                            CM_HELP_CONTENTS => "Help contents will migrate from WinHelp soon.",

                            CM_HELP_USING => "Using Help placeholder.",

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

fn set_checkbox(hwnd: HWND, control_id: i32, checked: bool) {
    let value = if checked { BST_CHECKED } else { BST_UNCHECKED };

    unsafe {
        let _ = SendDlgItemMessageW(
            hwnd,
            control_id,
            BM_SETCHECK,
            WPARAM(value as usize),
            LPARAM(0),
        );
    }
}

fn get_checkbox(hwnd: HWND, control_id: i32) -> bool {
    unsafe {
        SendDlgItemMessageW(hwnd, control_id, BM_GETCHECK, WPARAM(0), LPARAM(0)).0 as u32
            == BST_CHECKED
    }
}

fn dialog_result(result: isize) -> Result<()> {
    if result == -1 {
        Err(Error::from_win32())
    } else {
        Ok(())
    }
}

unsafe extern "system" fn options_dialog_proc(
    hwnd: HWND,

    message: u32,

    wparam: WPARAM,

    lparam: LPARAM,
) -> isize {
    match message {
        WM_INITDIALOG => {
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, lparam.0 as isize);

            let state_ptr = lparam.0 as *mut WindowState;

            if let Some(state) = state_ptr.as_ref() {
                set_checkbox(hwnd, ID_OPT_CARDS, state.options_show_cheat_cards);

                set_checkbox(hwnd, ID_OPT_STACK, state.options_show_cheat_stack);

                set_checkbox(hwnd, ID_OPT_SCORES, state.options_show_cheat_scores);

                set_checkbox(hwnd, ID_OPT_SAVE_EXIT, state.options_save_on_exit);
            }

            1
        }

        WM_COMMAND => {
            let command_id = (wparam.0 & 0xFFFF) as i32;

            match command_id {
                id if id == IDOK.0 => {
                    let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut WindowState;

                    if let Some(state) = state_ptr.as_mut() {
                        state.options_show_cheat_cards = get_checkbox(hwnd, ID_OPT_CARDS);

                        state.options_show_cheat_stack = get_checkbox(hwnd, ID_OPT_STACK);

                        state.options_show_cheat_scores = get_checkbox(hwnd, ID_OPT_SCORES);

                        state.options_save_on_exit = get_checkbox(hwnd, ID_OPT_SAVE_EXIT);
                    }

                    let _ = EndDialog(hwnd, command_id as isize);

                    1
                }

                id if id == IDCANCEL.0 => {
                    let _ = EndDialog(hwnd, command_id as isize);

                    1
                }

                _ => 0,
            }
        }

        _ => 0,
    }
}

unsafe extern "system" fn about_dialog_proc(
    hwnd: HWND,

    message: u32,

    wparam: WPARAM,

    _lparam: LPARAM,
) -> isize {
    match message {
        WM_INITDIALOG => {
            let _ = SetDlgItemTextW(
                hwnd,
                ID_ABOUT_TITLE,
                PCWSTR(wide_string("Stop the Bus").as_ptr()),
            );

            let _ = SetDlgItemTextW(
                hwnd,
                ID_ABOUT_COPYRIGHT,
                PCWSTR(wide_string("Copyright (c) Martin Davidson - 1994").as_ptr()),
            );

            let _ = SetDlgItemTextW(
                hwnd,
                ID_ABOUT_VERSION,
                PCWSTR(wide_string("Version 2.0.0").as_ptr()),
            );

            let _ = SetDlgItemTextW(
                hwnd,
                ID_ABOUT_RELEASE,
                PCWSTR(wide_string("Modernization build: 2025-09-22").as_ptr()),
            );

            let _ = SetDlgItemTextW(
                hwnd,
                ID_ABOUT_LICENSE_NAME,
                PCWSTR(wide_string(LICENSE_NAME).as_ptr()),
            );

            let _ = SetDlgItemTextW(
                hwnd,
                ID_ABOUT_LICENSE_COMPANY,
                PCWSTR(wide_string(LICENSE_COMPANY).as_ptr()),
            );

            let _ = SetDlgItemTextW(
                hwnd,
                ID_ABOUT_ADDRESS,
                PCWSTR(wide_string(LICENSE_ADDRESS).as_ptr()),
            );

            1
        }

        WM_COMMAND => {
            let command_id = (wparam.0 & 0xFFFF) as i32;

            if command_id == IDOK.0 || command_id == IDCANCEL.0 {
                let _ = EndDialog(hwnd, command_id as isize);

                1
            } else {
                0
            }
        }

        _ => 0,
    }
}

fn register_cheat_window_classes(instance: HINSTANCE, icon: HICON) -> Result<()> {
    register_single_cheat_class(
        instance,
        CHEAT_CARDS_CLASS,
        icon,
        Some(cheat_cards_wnd_proc),
    )?;

    register_single_cheat_class(
        instance,
        CHEAT_STACK_CLASS,
        icon,
        Some(cheat_stack_wnd_proc),
    )?;

    register_single_cheat_class(
        instance,
        CHEAT_SCORES_CLASS,
        icon,
        Some(cheat_scores_wnd_proc),
    )?;

    Ok(())
}

fn register_single_cheat_class(
    instance: HINSTANCE,

    class_name: PCWSTR,

    icon: HICON,

    wnd_proc: WNDPROC,
) -> Result<()> {
    let cursor = unsafe { LoadCursorW(None, windows::Win32::UI::WindowsAndMessaging::IDC_ARROW)? };

    let class = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,

        style: CS_HREDRAW | CS_VREDRAW,

        lpfnWndProc: wnd_proc,

        cbClsExtra: 0,

        cbWndExtra: 0,

        hInstance: instance,

        hIcon: icon,

        hCursor: cursor,

        hbrBackground: HBRUSH::default(),

        lpszMenuName: PCWSTR::null(),

        lpszClassName: class_name,

        hIconSm: icon,
    };

    match unsafe { RegisterClassExW(&class) } {
        0 => Err(Error::from_win32()),

        _ => Ok(()),
    }
}

fn create_cheat_cards_window(state: &mut WindowState) -> Result<HWND> {
    create_cheat_window(
        state,
        CHEAT_CARDS_CLASS,
        w!("Cards Cheat"),
        state.cheat_cards_pos,
        CHEAT_CARDS_SIZE,
    )
}

fn create_cheat_stack_window(state: &mut WindowState) -> Result<HWND> {
    create_cheat_window(
        state,
        CHEAT_STACK_CLASS,
        w!("Stack Cheat"),
        state.cheat_stack_pos,
        CHEAT_STACK_SIZE,
    )
}

fn create_cheat_scores_window(state: &mut WindowState) -> Result<HWND> {
    create_cheat_window(
        state,
        CHEAT_SCORES_CLASS,
        w!("Scores Cheat"),
        state.cheat_scores_pos,
        CHEAT_SCORES_SIZE,
    )
}

fn create_cheat_window(
    state: &mut WindowState,

    class_name: PCWSTR,

    title: PCWSTR,

    position: (i32, i32),

    size: (i32, i32),
) -> Result<HWND> {
    let owner = state.main_hwnd.unwrap_or(HWND::default());

    let hwnd = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE(WS_EX_TOOLWINDOW.0),
            class_name,
            title,
            WINDOW_STYLE(WS_POPUP.0 | WS_CAPTION.0 | WS_SYSMENU.0 | WS_VISIBLE.0),
            position.0,
            position.1,
            size.0,
            size.1,
            owner,
            HMENU::default(),
            state.module_instance,
            Some(state as *mut _ as *mut c_void),
        )?
    };

    unsafe {
        let _ = ShowWindow(hwnd, SW_SHOWDEFAULT);
    }

    Ok(hwnd)
}

unsafe extern "system" fn cheat_cards_wnd_proc(
    hwnd: HWND,

    message: u32,

    wparam: WPARAM,

    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_CREATE => {
            let create = unsafe { &*(lparam.0 as *const CREATESTRUCTW) };

            unsafe { SetWindowLongPtrW(hwnd, GWLP_USERDATA, create.lpCreateParams as isize) };

            LRESULT(0)
        }

        WM_MOVE => {
            if let Some(state) = unsafe { cheat_window_state_mut(hwnd) } {
                if let Some(pos) = unsafe { window_screen_position(hwnd) } {
                    state.cheat_cards_pos = pos;

                    state.persist_cheat_settings();
                }
            }

            LRESULT(0)
        }

        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();

            let hdc = unsafe { BeginPaint(hwnd, &mut ps) };

            if let Some(state) = unsafe { cheat_window_state_ref(hwnd) } {
                unsafe { paint_cheat_cards(state, hwnd, hdc) };
            }

            unsafe {
                let _ = EndPaint(hwnd, &ps);
            };

            LRESULT(0)
        }

        WM_DESTROY => {
            if let Some(state) = unsafe { cheat_window_state_mut(hwnd) } {
                state.cheat_cards_window = None;
            }

            unsafe { SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0) };

            LRESULT(0)
        }

        _ => unsafe { DefWindowProcW(hwnd, message, wparam, lparam) },
    }
}

unsafe extern "system" fn cheat_stack_wnd_proc(
    hwnd: HWND,

    message: u32,

    wparam: WPARAM,

    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_CREATE => {
            let create = unsafe { &*(lparam.0 as *const CREATESTRUCTW) };

            unsafe { SetWindowLongPtrW(hwnd, GWLP_USERDATA, create.lpCreateParams as isize) };

            LRESULT(0)
        }

        WM_MOVE => {
            if let Some(state) = unsafe { cheat_window_state_mut(hwnd) } {
                if let Some(pos) = unsafe { window_screen_position(hwnd) } {
                    state.cheat_stack_pos = pos;

                    state.persist_cheat_settings();
                }
            }

            LRESULT(0)
        }

        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();

            let hdc = unsafe { BeginPaint(hwnd, &mut ps) };

            if let Some(state) = unsafe { cheat_window_state_ref(hwnd) } {
                unsafe { paint_cheat_stack(state, hwnd, hdc) };
            }

            unsafe {
                let _ = EndPaint(hwnd, &ps);
            };

            LRESULT(0)
        }

        WM_DESTROY => {
            if let Some(state) = unsafe { cheat_window_state_mut(hwnd) } {
                state.cheat_stack_window = None;
            }

            unsafe { SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0) };

            LRESULT(0)
        }

        _ => unsafe { DefWindowProcW(hwnd, message, wparam, lparam) },
    }
}

unsafe extern "system" fn cheat_scores_wnd_proc(
    hwnd: HWND,

    message: u32,

    wparam: WPARAM,

    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_CREATE => {
            let create = unsafe { &*(lparam.0 as *const CREATESTRUCTW) };

            unsafe { SetWindowLongPtrW(hwnd, GWLP_USERDATA, create.lpCreateParams as isize) };

            LRESULT(0)
        }

        WM_MOVE => {
            if let Some(state) = unsafe { cheat_window_state_mut(hwnd) } {
                if let Some(pos) = unsafe { window_screen_position(hwnd) } {
                    state.cheat_scores_pos = pos;

                    state.persist_cheat_settings();
                }
            }

            LRESULT(0)
        }

        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();

            let hdc = unsafe { BeginPaint(hwnd, &mut ps) };

            if let Some(state) = unsafe { cheat_window_state_ref(hwnd) } {
                unsafe { paint_cheat_scores(state, hwnd, hdc) };
            }

            unsafe {
                let _ = EndPaint(hwnd, &ps);
            };

            LRESULT(0)
        }

        WM_DESTROY => {
            if let Some(state) = unsafe { cheat_window_state_mut(hwnd) } {
                state.cheat_scores_window = None;
            }

            unsafe { SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0) };

            LRESULT(0)
        }

        _ => unsafe { DefWindowProcW(hwnd, message, wparam, lparam) },
    }
}

unsafe fn cheat_window_state_ref(hwnd: HWND) -> Option<&'static WindowState> {
    let ptr = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) } as *const WindowState;

    unsafe { ptr.as_ref() }
}

unsafe fn cheat_window_state_mut(hwnd: HWND) -> Option<&'static mut WindowState> {
    let ptr = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) } as *mut WindowState;

    unsafe { ptr.as_mut() }
}

fn registry_read_dword(value_name: PCWSTR) -> Option<u32> {
    let mut data: u32 = 0;

    let mut data_size = size_of::<u32>() as u32;

    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            REGISTRY_SUBKEY,
            value_name,
            RRF_RT_REG_DWORD,
            None,
            Some(std::ptr::addr_of_mut!(data) as *mut c_void),
            Some(&mut data_size),
        )
    };

    if status == ERROR_SUCCESS {
        Some(data)
    } else {
        None
    }
}

fn registry_write_dword(value_name: PCWSTR, value: u32) {
    let _ = unsafe {
        RegSetKeyValueW(
            HKEY_CURRENT_USER,
            REGISTRY_SUBKEY,
            value_name,
            REG_DWORD.0,
            Some(std::ptr::addr_of!(value) as *const c_void),
            size_of::<u32>() as u32,
        )
    };
}

fn registry_read_bool(value_name: PCWSTR) -> Option<bool> {
    registry_read_dword(value_name).map(|value| value != 0)
}

fn registry_write_bool(value_name: PCWSTR, value: bool) {
    registry_write_dword(value_name, value as u32);
}

fn registry_read_point(x_name: PCWSTR, y_name: PCWSTR) -> Option<(i32, i32)> {
    match (registry_read_dword(x_name), registry_read_dword(y_name)) {
        (Some(x), Some(y)) => Some((x as i32, y as i32)),

        _ => None,
    }
}

fn registry_write_point(x_name: PCWSTR, y_name: PCWSTR, value: (i32, i32)) {
    registry_write_dword(x_name, value.0 as u32);

    registry_write_dword(y_name, value.1 as u32);
}

unsafe fn window_screen_position(hwnd: HWND) -> Option<(i32, i32)> {
    if !IsWindowVisible(hwnd).as_bool() {
        return None;
    }

    let mut rect = RECT::default();

    if GetWindowRect(hwnd, &mut rect).is_ok() {
        if rect.left <= -32000 || rect.top <= -32000 {
            None
        } else {
            Some((rect.left, rect.top))
        }
    } else {
        None
    }
}

unsafe fn fill_cheat_background(hwnd: HWND, hdc: HDC) {
    let mut rect = RECT::default();

    if GetClientRect(hwnd, &mut rect).is_ok() {
        let brush = CreateSolidBrush(COLORREF(0x00C0C0C0));

        let _ = FillRect(hdc, &rect, brush);

        let _ = DeleteObject(brush);
    }
}

unsafe fn paint_cheat_cards(state: &WindowState, hwnd: HWND, hdc: HDC) {
    fill_cheat_background(hwnd, hdc);

    let mem_dc = CreateCompatibleDC(hdc);

    if mem_dc.0.is_null() {
        return;
    }

    SetBkColor(hdc, COLORREF(0x00C0C0C0));

    SetTextColor(hdc, COLORREF(0x00000000));

    for (player_index, hand) in state.game.hands.iter().enumerate().skip(1) {
        let player_number = player_index + 1;

        for (slot, card) in hand.iter().enumerate() {
            let bitmap = card
                .and_then(|id| state.card_bitmap(id))
                .unwrap_or(state.card_cross);

            let x = 50 + 51 * (slot as i32 + 1);

            let y = 57 * (player_number as i32) - 108;

            draw_small_bitmap(hdc, mem_dc, bitmap, x, y);
        }

        let label = format!("Player {}:", player_number);

        draw_text(hdc, 10, 57 * (player_number as i32) - 100, &label);
    }

    let _ = DeleteDC(mem_dc);
}

unsafe fn paint_cheat_stack(state: &WindowState, hwnd: HWND, hdc: HDC) {
    fill_cheat_background(hwnd, hdc);

    let mem_dc = CreateCompatibleDC(hdc);

    if mem_dc.0.is_null() {
        return;
    }

    SetBkColor(hdc, COLORREF(0x00C0C0C0));

    SetTextColor(hdc, COLORREF(0x00000000));

    let base_index = state.game.stack_index();

    for i in 0..CHEAT_STACK_PREVIEW {
        let deck_index = base_index + i + 1;

        let bitmap = if deck_index < DECK_SIZE {
            state
                .game
                .deck
                .get(deck_index)
                .copied()
                .and_then(|id| state.card_bitmap(id))
                .unwrap_or(state.card_cross)
        } else {
            state.card_cross
        };

        let x = 50 * (i as i32 + 1) - 45;

        let y = 30;

        draw_small_bitmap(hdc, mem_dc, bitmap, x, y);
    }

    let label = format!("Stack pointer: {}", base_index + 1);

    draw_text(hdc, 10, 5, &label);

    let _ = DeleteDC(mem_dc);
}

unsafe fn paint_cheat_scores(state: &WindowState, hwnd: HWND, hdc: HDC) {
    fill_cheat_background(hwnd, hdc);

    SetBkColor(hdc, COLORREF(0x00C0C0C0));

    SetTextColor(hdc, COLORREF(0x00000000));

    for (index, score) in state.game.round_scores.iter().enumerate() {
        let text = format!("Player {}: {}", index + 1, score);

        let y = 20 * (index as i32 + 1) - 16;

        draw_text(hdc, 20, y, &text);
    }
}

unsafe fn load_bitmap(instance: HINSTANCE, id: u16) -> Result<HBITMAP> {
    match load_bitmap_from_resource(instance, id) {
        Ok(bitmap) => Ok(bitmap),

        Err(_) => {
            let handle = LoadBitmapW(instance, make_int_resource(id));

            if handle.0.is_null() {
                Err(Error::from_win32())
            } else {
                Ok(handle)
            }
        }
    }
}

unsafe fn load_bitmap_from_resource(instance: HINSTANCE, id: u16) -> Result<HBITMAP> {
    let resource = FindResourceW(instance, make_int_resource(id), RT_BITMAP);

    if resource.is_invalid() {
        return Err(Error::from_win32());
    }

    let size = SizeofResource(instance, resource);

    if size == 0 {
        return Err(Error::from_win32());
    }

    let handle = LoadResource(instance, resource)?;

    let data_ptr = LockResource(handle) as *const u8;

    if data_ptr.is_null() {
        return Err(Error::from_win32());
    }

    let data = unsafe { slice::from_raw_parts(data_ptr, size as usize) };

    create_dib_from_resource(data)
}

fn create_dib_from_resource(data: &[u8]) -> Result<HBITMAP> {
    if data.len() < size_of::<BITMAPINFOHEADER>() {
        return Err(Error::from_win32());
    }

    let mut header = BITMAPINFOHEADER::default();

    unsafe {
        copy_nonoverlapping(
            data.as_ptr() as *const BITMAPINFOHEADER,
            &mut header as *mut BITMAPINFOHEADER,
            1,
        );
    }

    if header.biCompression != BI_RGB.0 {
        return Err(Error::from_win32());
    }

    let width = header.biWidth as i32;

    let height = header.biHeight as i32;

    let abs_height = height.abs() as usize;

    let bit_count = header.biBitCount as usize;

    if width == 0 || abs_height == 0 || bit_count == 0 {
        return Err(Error::from_win32());
    }

    let palette_entries = if bit_count <= 8 {
        let used = header.biClrUsed as usize;

        if used != 0 {
            used
        } else {
            1 << bit_count
        }
    } else {
        0
    };

    let palette_bytes = palette_entries * size_of::<RGBQUAD>();

    let bits_offset = header.biSize as usize + palette_bytes;

    if data.len() < bits_offset {
        return Err(Error::from_win32());
    }

    let stride = ((width.abs() as usize * bit_count + 31) / 32) * 4;

    let bits = &data[bits_offset..];

    if bits.len() < stride * abs_height {
        return Err(Error::from_win32());
    }

    let palette = if palette_entries > 0 {
        &data[header.biSize as usize..header.biSize as usize + palette_bytes]
    } else {
        &[]
    };

    let width_usize = width.abs() as usize;

    let mut output = vec![0u8; abs_height * width_usize * 4];

    for row in 0..abs_height {
        let src_row = if height > 0 {
            abs_height - 1 - row
        } else {
            row
        };

        let row_data = &bits[src_row * stride..(src_row + 1) * stride];

        for col in 0..width_usize {
            let (b, g, r, a) = match bit_count {
                1 => {
                    let byte = row_data[col / 8];

                    let mask = 0x80 >> (col % 8);

                    let idx = if byte & mask == 0 { 0 } else { 1 };

                    let base = idx * 4;

                    (
                        palette.get(base).copied().unwrap_or(0),
                        palette.get(base + 1).copied().unwrap_or(0),
                        palette.get(base + 2).copied().unwrap_or(0),
                        0xFF,
                    )
                }

                4 => {
                    let byte = row_data[col / 2];

                    let idx = if col % 2 == 0 {
                        (byte >> 4) as usize
                    } else {
                        (byte & 0x0F) as usize
                    };

                    let base = idx * 4;

                    (
                        palette.get(base).copied().unwrap_or(0),
                        palette.get(base + 1).copied().unwrap_or(0),
                        palette.get(base + 2).copied().unwrap_or(0),
                        0xFF,
                    )
                }

                8 => {
                    let idx = row_data[col] as usize;

                    let base = idx * 4;

                    (
                        palette.get(base).copied().unwrap_or(0),
                        palette.get(base + 1).copied().unwrap_or(0),
                        palette.get(base + 2).copied().unwrap_or(0),
                        0xFF,
                    )
                }

                24 => {
                    let base = col * 3;

                    (row_data[base], row_data[base + 1], row_data[base + 2], 0xFF)
                }

                32 => {
                    let base = col * 4;

                    (
                        row_data[base],
                        row_data[base + 1],
                        row_data[base + 2],
                        row_data[base + 3],
                    )
                }

                _ => return Err(Error::from_win32()),
            };

            let dest_index = (row * width_usize + col) * 4;

            output[dest_index] = b;

            output[dest_index + 1] = g;

            output[dest_index + 2] = r;

            output[dest_index + 3] = a;
        }
    }

    let mut info = BITMAPINFOHEADER::default();

    info.biSize = size_of::<BITMAPINFOHEADER>() as u32;

    info.biWidth = width;

    info.biHeight = -(abs_height as i32);

    info.biPlanes = 1;

    info.biBitCount = 32;

    info.biCompression = BI_RGB.0;

    info.biSizeImage = output.len() as u32;

    let mut bmi = BITMAPINFO {
        bmiHeader: info,

        bmiColors: [RGBQUAD::default(); 1],
    };

    let mut bits_ptr: *mut std::ffi::c_void = std::ptr::null_mut();

    let bitmap =
        unsafe { CreateDIBSection(None, &mut bmi, DIB_RGB_COLORS, &mut bits_ptr, None, 0)? };

    unsafe {
        copy_nonoverlapping(output.as_ptr(), bits_ptr as *mut u8, output.len());
    }

    Ok(bitmap)
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

fn show_error(hwnd: HWND, message: &str) {
    let text = wide_string(message);

    let title = wide_string("Stop the Bus");

    let _ = unsafe {
        MessageBoxW(
            hwnd,
            PCWSTR(text.as_ptr()),
            PCWSTR(title.as_ptr()),
            MB_OK | MB_ICONEXCLAMATION,
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
