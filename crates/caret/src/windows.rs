/// Windows 平台的光标位置获取实现
/// 
/// 使用混合策略以支持不同类型的应用程序：
/// 1. GetGUIThreadInfo - 快速，支持标准 Windows 控件
/// 2. UI Automation - 较慢但兼容性好，支持 Chrome、VS Code 等现代应用

use super::Position;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, SystemTime};
use windows::{
    core::*,
    Win32::{
        Foundation::{POINT, S_OK, S_FALSE, HWND},
        Graphics::Gdi::ClientToScreen,
        System::Com::*,
        System::Ole::{SafeArrayAccessData, SafeArrayUnaccessData},
        UI::{
            Accessibility::*,
            WindowsAndMessaging::{
                GetForegroundWindow, GetGUIThreadInfo, GetWindowThreadProcessId, GUITHREADINFO,
                GetMessageW, TranslateMessage, DispatchMessageW, MSG,
                WINEVENT_OUTOFCONTEXT, EVENT_OBJECT_LOCATIONCHANGE,
            },
        },
    },
};

// VARIANT 类型定义 (用于 COM 互操作)
#[repr(C)]
#[derive(Clone)]
struct VARIANT {
    data: [u8; 24], // VARIANT 的实际大小
}

// MSAA 相关的常量
const OBJID_CARET: i32 = -8;

// HWINEVENTHOOK 类型定义
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
struct HWINEVENTHOOK(isize);

impl HWINEVENTHOOK {
    fn is_invalid(&self) -> bool {
        self.0 == 0
    }
}

// Windows API 函数声明
unsafe extern "system" {
    fn SetWinEventHook(
        event_min: u32,
        event_max: u32,
        hmod_win_event_proc: usize,
        pfn_win_event_proc: usize,
        id_process: u32,
        id_thread: u32,
        dw_flags: u32,
    ) -> HWINEVENTHOOK;

    fn UnhookWinEvent(h_win_event_hook: HWINEVENTHOOK) -> i32;

    fn VariantInit(pvar: *mut VARIANT);
    fn VariantClear(pvar: *mut VARIANT) -> HRESULT;
    
    fn AccessibleObjectFromEvent(
        hwnd: HWND,
        dw_id: u32,
        dw_child_id: u32,
        pp_acc: *mut *mut core::ffi::c_void,
        pvar_child: *mut VARIANT,
    ) -> HRESULT;
}

// IAccessible::accLocation 的直接调用
// COM vtable 中的第 22 个方法是 accLocation
type AccLocationFn = unsafe extern "system" fn(
    this: *mut core::ffi::c_void,
    px_left: *mut i32,
    py_top: *mut i32,
    pcx_width: *mut i32,
    pcy_height: *mut i32,
    var_child: VARIANT,
) -> HRESULT;

unsafe fn call_acc_location(
    p_acc: *mut core::ffi::c_void,
    left: *mut i32,
    top: *mut i32,
    width: *mut i32,
    height: *mut i32,
    var_child: VARIANT,
) -> HRESULT {
    unsafe {
        let vtable = *(p_acc as *mut *mut usize);
        let acc_location_fn: AccLocationFn = std::mem::transmute(*(vtable.add(22)));
        acc_location_fn(p_acc, left, top, width, height, var_child)
    }
}

/// MSAA 光标位置缓存
/// 存储最近一次通过 SetWinEventHook + MSAA 获取到的光标位置
struct MsaaCaretCache {
    position: Position,
    timestamp: SystemTime,
}

/// 全局 MSAA 光标位置缓存
static MSAA_CARET_CACHE: OnceLock<Mutex<Option<MsaaCaretCache>>> = OnceLock::new();

/// 获取当前活动窗口的光标位置
/// 
/// 按照以下优先级尝试：
/// 1. GetGUIThreadInfo（适用于 Sublime Text、Notepad 等）
/// 2. UI Automation（适用于 Chrome、Edge、VS Code 等）
/// 3. MSAA + SetWinEventHook（兜底策略，支持传统 MSAA 应用）
/// 4. 失败则返回 None
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
/// 
/// 适用于使用标准 Windows 控件的应用（如 Sublime Text、Notepad 等）
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

            // 检查光标矩形是否完全为空（所有字段都为 0）
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
/// 
/// 适用于使用自定义渲染的现代应用（如 Chrome、VS Code 等）
#[tracing::instrument(name = "get_from_ui_automation")]
fn try_get_from_ui_automation() -> Option<Position> {
    unsafe {
        // 初始化 COM（S_OK 和 S_FALSE 都表示成功）
        let hr = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        if hr != S_OK && hr != S_FALSE {
            tracing::error!("COM 初始化失败");
            return None;
        }

        let result = (|| -> windows::core::Result<Position> {
            // 创建 UI Automation 实例
            let automation: IUIAutomation =
                CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER)
                    .map_err(|e| {
                        tracing::error!(error = ?e, "创建 UI Automation 实例失败");
                        e
                    })?;

            // 获取焦点元素
            let focused_element = automation.GetFocusedElement()
                .map_err(|e| {
                    tracing::debug!(error = ?e, "获取焦点元素失败");
                    e
                })?;

            // 尝试获取 TextPattern
            let text_pattern: IUIAutomationTextPattern =
                focused_element.GetCurrentPatternAs(UIA_TextPatternId)
                    .map_err(|e| {
                        tracing::debug!(error = ?e, "获取 TextPattern 失败（当前应用可能不支持文本光标）");
                        e
                    })?;

            // 获取选区（光标位置）
            let selection_array = text_pattern.GetSelection()
                .map_err(|e| {
                    tracing::debug!(error = ?e, "获取选区失败");
                    e
                })?;
                
            let length = selection_array.Length()
                .map_err(|e| {
                    tracing::debug!(error = ?e, "获取选区长度失败");
                    e
                })?;

            if length > 0 {
                let selection_range = selection_array.GetElement(0)
                    .map_err(|e| {
                        tracing::debug!(error = ?e, "获取选区元素失败");
                        e
                    })?;

                // 获取选区的边界矩形（返回 *mut SAFEARRAY）
                let rects_array = selection_range.GetBoundingRectangles()
                    .map_err(|e| {
                        tracing::debug!(error = ?e, "获取边界矩形失败");
                        e
                    })?;

                // 正确访问 SAFEARRAY 数据
                let mut data_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
                if SafeArrayAccessData(rects_array, &mut data_ptr).is_ok() {
                    let rect_data = std::slice::from_raw_parts(data_ptr as *const f64, 4);
                    let x = rect_data[0] as i32;
                    let y = rect_data[1] as i32;
                    let width = rect_data[2] as i32;
                    let height = rect_data[3] as i32;

                    // 释放 SAFEARRAY 访问
                    let _ = SafeArrayUnaccessData(rects_array);

                    tracing::debug!(x, y, width, height, "UI Automation 返回的原始坐标");

                    // 验证坐标有效性：(0, 0) 通常表示无效位置
                    // 在 VSCode 等应用中，UI Automation 可能返回 (0, 0) 表示光标位置未正确获取
                    if x == 0 && y == 0 {
                        tracing::debug!("检测到无效坐标 (0, 0)，可能是应用未正确暴露光标位置");
                        return Err(Error::from(windows::core::HRESULT(-1)));
                    }

                    return Ok(Position {
                        x,
                        y: y + height, // 光标底部位置
                    });
                } else {
                    tracing::debug!("访问 SAFEARRAY 数据失败");
                }
            } else {
                tracing::debug!("选区为空（没有文本选择）");
            }

            Err(Error::from(windows::core::HRESULT(-1)))
        })();

        CoUninitialize();

        result.ok()
    }
}

/// 策略 3: 从 MSAA 钩子缓存中获取光标位置
/// 
/// 适用于实现了 MSAA 接口的传统应用（如 Office、WinForms、WPF 等）
#[tracing::instrument(name = "get_from_msaa_hook")]
fn try_get_from_msaa_hook() -> Option<Position> {
    // 确保钩子线程已启动
    ensure_msaa_hook_started();

    // 从缓存中读取最近的光标位置
    let cache = MSAA_CARET_CACHE.get()?;
    let cache_guard = cache.lock().ok()?;
    
    if let Some(cached) = cache_guard.as_ref() {
        // 检查缓存是否还新鲜（2秒内）
        if let Ok(elapsed) = cached.timestamp.elapsed() {
            if elapsed < Duration::from_secs(2) {
                tracing::debug!(
                    x = cached.position.x,
                    y = cached.position.y,
                    age_ms = elapsed.as_millis(),
                    "使用 MSAA 缓存的光标位置"
                );
                return Some(cached.position);
            } else {
                tracing::debug!(age_ms = elapsed.as_millis(), "MSAA 缓存已过期");
            }
        }
    }

    None
}

/// 确保 MSAA 钩子线程已启动
fn ensure_msaa_hook_started() {
    use std::sync::Once;
    static START_HOOK: Once = Once::new();

    START_HOOK.call_once(|| {
        // 初始化全局缓存
        let _ = MSAA_CARET_CACHE.set(Mutex::new(None));

        // 启动后台线程监听 MSAA 事件
        std::thread::spawn(|| {
            msaa_hook_thread();
        });

        tracing::info!("MSAA 钩子线程已启动");
    });
}

/// MSAA 钩子线程主函数
fn msaa_hook_thread() {
    unsafe {
        // 初始化 COM
        let hr = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        if hr != S_OK && hr != S_FALSE {
            tracing::error!("MSAA 钩子线程：COM 初始化失败");
            return;
        }

        // 设置 WinEvent 钩子监听光标位置变化
        let hook = SetWinEventHook(
            EVENT_OBJECT_LOCATIONCHANGE,  // eventMin
            EVENT_OBJECT_LOCATIONCHANGE,  // eventMax
            0,                            // hmodWinEventProc
            win_event_proc as usize,      // 回调函数
            0,                            // idProcess (0 = 所有进程)
            0,                            // idThread (0 = 所有线程)
            WINEVENT_OUTOFCONTEXT,        // dwFlags (异步调用)
        );

        if hook.is_invalid() {
            tracing::error!("设置 WinEvent 钩子失败");
            CoUninitialize();
            return;
        }

        tracing::info!("MSAA WinEvent 钩子已设置，开始监听光标位置变化");

        // 消息循环
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        // 清理
        let _ = UnhookWinEvent(hook);
        CoUninitialize();
        tracing::info!("MSAA 钩子线程已退出");
    }
}

/// WinEvent 回调函数
/// 
/// 当系统中发生光标位置变化事件时被调用
unsafe extern "system" fn win_event_proc(
    _h_win_event_hook: HWINEVENTHOOK,
    event: u32,
    hwnd: HWND,
    id_object: i32,
    id_child: i32,
    _dw_event_thread: u32,
    _dwms_event_time: u32,
) {
    // 只处理光标位置变化事件
    if event != EVENT_OBJECT_LOCATIONCHANGE || id_object != OBJID_CARET as i32 {
        return;
    }

    unsafe {
        // 获取 IAccessible 对象
        let mut p_acc: *mut core::ffi::c_void = std::ptr::null_mut();
        let mut var_child: VARIANT = std::mem::zeroed();
        VariantInit(&mut var_child as *mut VARIANT);

        let hr = AccessibleObjectFromEvent(
            hwnd,
            id_object as u32,
            id_child as u32,
            &mut p_acc,
            &mut var_child,
        );

        if hr.is_err() || p_acc.is_null() {
            let _ = VariantClear(&mut var_child as *mut VARIANT);
            return;
        }

        // 获取光标位置
        let mut left: i32 = 0;
        let mut top: i32 = 0;
        let mut width: i32 = 0;
        let mut height: i32 = 0;

        let location_result = call_acc_location(
            p_acc,
            &mut left,
            &mut top,
            &mut width,
            &mut height,
            var_child.clone(),
        );

        // 清理资源
        let _ = VariantClear(&mut var_child as *mut VARIANT);

        if location_result.is_err() {
            return;
        }

        // 验证坐标有效性
        if left > 0 && top > 0 && width > 0 && height > 0 {
            let position = Position {
                x: left,
                y: top + height, // 光标底部位置
            };

            // 更新全局缓存
            if let Some(cache) = MSAA_CARET_CACHE.get() {
                if let Ok(mut cache_guard) = cache.lock() {
                    *cache_guard = Some(MsaaCaretCache {
                        position,
                        timestamp: SystemTime::now(),
                    });

                    tracing::debug!(
                        x = position.x,
                        y = position.y,
                        width,
                        height,
                        "MSAA 检测到光标位置变化"
                    );
                }
            }
        }
    }
}
