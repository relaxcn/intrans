/// Windows 平台的光标位置获取实现
/// 
/// 使用混合策略以支持不同类型的应用程序：
/// 
/// 1. **GetGUIThreadInfo** - 快速，支持标准 Windows 控件
///    - 适用：Sublime Text、Notepad、记事本等传统桌面应用
///    - 优点：调用开销极低，响应迅速
///    - 限制：仅支持使用标准 Win32 控件的应用
/// 
/// 2. **UI Automation** - 兼容性好，支持现代应用
///    - 适用：Chrome、Edge、VS Code、Electron 应用等
///    - 优点：支持自绘控件的现代应用
///    - 限制：调用较慢，需要 COM 初始化
/// 
/// 3. **MSAA + SetWinEventHook** - 兜底策略，支持传统辅助功能应用
///    - 适用：Microsoft Office、WinForms、WPF 等实现了 MSAA 的应用
///    - 优点：事件驱动，自动缓存最近的光标位置
///    - 限制：依赖应用实现 MSAA 接口，缓存有 2 秒有效期
/// 
/// ## 工作原理
/// 
/// 按优先级依次尝试上述三种策略，任一成功即返回结果。
/// MSAA 策略使用后台线程监听系统级光标位置变化事件，将最近的位置缓存在内存中。

use super::Position;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, SystemTime};
use windows::{
    core::*,
    Win32::{
        Foundation::{POINT, HWND, RPC_E_CHANGED_MODE},
        Graphics::Gdi::ClientToScreen,
        System::{
            Com::*,
            Ole::{SafeArrayAccessData, SafeArrayUnaccessData},
            Variant::{VARIANT, VariantClear},
        },
        UI::{
            Accessibility::*,
            WindowsAndMessaging::{
                GetForegroundWindow, GetGUIThreadInfo, GetWindowThreadProcessId, GUITHREADINFO,
                GetMessageW, TranslateMessage, DispatchMessageW, MSG,
            },
        },
    },
};

// 补充常量定义，以防 windows crate 版本差异导致找不到
const WINEVENT_OUTOFCONTEXT: u32 = 0x0000;
const EVENT_OBJECT_LOCATIONCHANGE: u32 = 0x800B;

/// MSAA 光标位置缓存
/// 存储最近一次通过 SetWinEventHook + MSAA 获取到的光标位置
struct MsaaCaretCache {
    position: Position,
    timestamp: SystemTime,
}

/// 全局 MSAA 光标位置缓存
static MSAA_CARET_CACHE: OnceLock<Mutex<Option<MsaaCaretCache>>> = OnceLock::new();

/// 获取当前活动窗口的光标位置
#[tracing::instrument(name = "get_caret_position")]
pub fn get_position() -> Option<Position> {
    // 策略 1: 尝试使用 GetGUIThreadInfo（快速，支持标准控件）
    if let Some(pos) = try_get_from_gui_thread_info() {
        tracing::info!(x = pos.x, y = pos.y, method = "GetGUIThreadInfo", "成功获取光标位置");
        return Some(pos);
    }

    // 策略 2: 回退到 UI Automation（较慢但兼容性更好，支持 Chrome 等）
    if let Some(pos) = try_get_from_ui_automation() {
        tracing::info!(x = pos.x, y = pos.y, method = "UI Automation", "成功获取光标位置");
        return Some(pos);
    }

    // 策略 3: 尝试从 MSAA 钩子缓存中获取（适用于传统 MSAA 应用）
    if let Some(pos) = try_get_from_msaa_hook() {
        tracing::info!(x = pos.x, y = pos.y, method = "MSAA Hook", "成功获取光标位置");
        return Some(pos);
    }

    // 策略 4: 都失败则返回 None
    tracing::warn!("所有光标获取方法都失败");
    None
}

/// 策略 1: 使用 GetGUIThreadInfo 获取光标位置
#[tracing::instrument(name = "get_from_gui_thread_info")]
fn try_get_from_gui_thread_info() -> Option<Position> {
    unsafe {
        let foreground_window = GetForegroundWindow();
        if foreground_window.0.is_null() {
            return None;
        }

        let mut process_id = 0;
        let thread_id = GetWindowThreadProcessId(foreground_window, Some(&mut process_id));

        let mut gui_info = GUITHREADINFO::default();
        gui_info.cbSize = std::mem::size_of::<GUITHREADINFO>() as u32;

        if GetGUIThreadInfo(thread_id, &mut gui_info).is_ok() {
            let caret_rect = gui_info.rcCaret;

            // 检查光标矩形是否完全为空
            if caret_rect.left == 0
                && caret_rect.top == 0
                && caret_rect.right == 0
                && caret_rect.bottom == 0
            {
                return None;
            }

            let mut point = POINT {
                x: caret_rect.left,
                y: caret_rect.bottom,
            };

            if !gui_info.hwndCaret.0.is_null() {
                if ClientToScreen(gui_info.hwndCaret, &mut point).as_bool() {
                    return Some(Position {
                        x: point.x,
                        y: point.y,
                    });
                }
            }
        }
    }
    None
}

/// 策略 2: 使用 UI Automation 获取光标位置
#[tracing::instrument(name = "get_from_ui_automation")]
fn try_get_from_ui_automation() -> Option<Position> {
    unsafe {
        // 初始化 COM
        let hr = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        if hr.is_err() && hr != RPC_E_CHANGED_MODE {
             tracing::debug!("COM 初始化失败: {:?}", hr);
        }
        
        // 使用 RAII 确保 CoUninitialize 被调用
        let should_uninitialize = hr.is_ok();

        let result = (|| -> windows::core::Result<Position> {
            let automation: IUIAutomation =
                CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER)?;

            let focused_element = automation.GetFocusedElement()?;

            // 尝试获取 TextPattern
            let text_pattern: IUIAutomationTextPattern =
                focused_element.GetCurrentPatternAs(UIA_TextPatternId)?;

            let selection_array = text_pattern.GetSelection()?;
            let length = selection_array.Length()?;

            if length > 0 {
                let selection_range = selection_array.GetElement(0)?;
                let rects_array = selection_range.GetBoundingRectangles()?;

                let mut data_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
                if SafeArrayAccessData(rects_array, &mut data_ptr).is_ok() {
                    let rect_data = std::slice::from_raw_parts(data_ptr as *const f64, 4);
                    let x = rect_data[0] as i32;
                    let y = rect_data[1] as i32;
                    let _width = rect_data[2] as i32;
                    let height = rect_data[3] as i32;

                    let _ = SafeArrayUnaccessData(rects_array);

                    if x == 0 && y == 0 {
                        return Err(Error::from_hresult(windows::core::HRESULT(-1)));
                    }

                    return Ok(Position {
                        x,
                        y: y + height,
                    });
                }
            }
            
            Err(Error::from_hresult(windows::core::HRESULT(-1)))
        })();

        if should_uninitialize {
            CoUninitialize();
        }

        result.ok()
    }
}

/// 策略 3: 从 MSAA 钩子缓存中获取光标位置
#[tracing::instrument(name = "get_from_msaa_hook")]
fn try_get_from_msaa_hook() -> Option<Position> {
    ensure_msaa_hook_started();

    let cache = MSAA_CARET_CACHE.get()?;
    let cache_guard = cache.lock().ok()?;
    
    if let Some(cached) = cache_guard.as_ref() {
        if let Ok(elapsed) = cached.timestamp.elapsed() {
            if elapsed < Duration::from_secs(2) {
                return Some(cached.position);
            }
        }
    }

    None
}

fn ensure_msaa_hook_started() {
    use std::sync::Once;
    static START_HOOK: Once = Once::new();

    START_HOOK.call_once(|| {
        let _ = MSAA_CARET_CACHE.set(Mutex::new(None));
        std::thread::spawn(msaa_hook_thread);
        tracing::info!("MSAA 钩子线程已启动");
    });
}

fn msaa_hook_thread() {
    unsafe {
        if CoInitializeEx(None, COINIT_APARTMENTTHREADED).is_err() {
            return;
        }

        let hook = SetWinEventHook(
            EVENT_OBJECT_LOCATIONCHANGE,
            EVENT_OBJECT_LOCATIONCHANGE,
            None,
            Some(win_event_proc),
            0,
            0,
            WINEVENT_OUTOFCONTEXT,
        );

        if hook.is_invalid() {
            CoUninitialize();
            return;
        }

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        let _ = UnhookWinEvent(hook);
        CoUninitialize();
    }
}

unsafe extern "system" fn win_event_proc(
    _h_win_event_hook: HWINEVENTHOOK,
    event: u32,
    hwnd: HWND,
    id_object: i32,
    id_child: i32,
    _dw_event_thread: u32,
    _dwms_event_time: u32,
) {
    // OBJID_CARET 在 Windows API 中定义为 -8 (0xFFFFFFF8)
    const OBJID_CARET_VAL: i32 = -8;
    
    if event != EVENT_OBJECT_LOCATIONCHANGE || id_object != OBJID_CARET_VAL {
        return;
    }

    let mut p_acc: Option<IAccessible> = None;
    let mut var_child = VARIANT::default();
    // VariantInit is not needed for default initialized VARIANT

    // 使用 windows crate 的 AccessibleObjectFromEvent
    let hr = unsafe { AccessibleObjectFromEvent(
        hwnd,
        id_object as u32,
        id_child as u32,
        &mut p_acc,
        &mut var_child,
    ) };

    if hr.is_err() || p_acc.is_none() {
        let _ = unsafe { VariantClear(&mut var_child) };
        return;
    }

    let acc = p_acc.unwrap();

    // 直接调用 IAccessible::accLocation
    let mut left = 0;
    let mut top = 0;
    let mut width = 0;
    let mut height = 0;

    let location_result = unsafe { acc.accLocation(
        &mut left,
        &mut top,
        &mut width,
        &mut height,
        &var_child,
    ) };

    let _ = unsafe { VariantClear(&mut var_child) };

    if location_result.is_ok() && left > 0 && top > 0 && width > 0 && height > 0 {
        let position = Position {
            x: left,
            y: top + height,
        };

        if let Some(cache) = MSAA_CARET_CACHE.get() {
            if let Ok(mut cache_guard) = cache.lock() {
                *cache_guard = Some(MsaaCaretCache {
                    position,
                    timestamp: SystemTime::now(),
                });
            }
        }
    }
}
