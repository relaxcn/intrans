/// Windows 平台的光标位置获取实现
/// 
/// 使用混合策略以支持不同类型的应用程序：
/// 1. GetGUIThreadInfo - 快速，支持标准 Windows 控件
/// 2. UI Automation - 较慢但兼容性好，支持 Chrome、VS Code 等现代应用

use super::Position;
use windows::{
    core::*,
    Win32::{
        Foundation::POINT,
        Graphics::Gdi::ClientToScreen,
        System::Com::*,
        UI::{
            Accessibility::*,
            WindowsAndMessaging::{
                GetForegroundWindow, GetGUIThreadInfo, GetWindowThreadProcessId, GUITHREADINFO,
            },
        },
    },
};

/// 获取当前活动窗口的光标位置
/// 
/// 按照以下优先级尝试：
/// 1. GetGUIThreadInfo（适用于 Sublime Text、Notepad 等）
/// 2. UI Automation（适用于 Chrome、Edge、VS Code 等）
/// 3. 失败则返回 None
pub fn get_position() -> Option<Position> {
    // 策略 1: 尝试使用 GetGUIThreadInfo（快速，支持标准控件）
    if let Some(pos) = try_get_from_gui_thread_info() {
        println!("✓ 使用 GetGUIThreadInfo 获取光标位置");
        return Some(pos);
    }

    // 策略 2: 回退到 UI Automation（较慢但兼容性更好，支持 Chrome 等）
    if let Some(pos) = try_get_from_ui_automation() {
        println!("✓ 使用 UI Automation 获取光标位置");
        return Some(pos);
    }

    // 策略 3: 都失败则返回 None
    println!("✗ 所有光标获取方法都失败");
    None
}

/// 策略 1: 使用 GetGUIThreadInfo 获取光标位置
/// 
/// 适用于使用标准 Windows 控件的应用（如 Sublime Text、Notepad 等）
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
fn try_get_from_ui_automation() -> Option<Position> {
    unsafe {
        // 初始化 COM
        if CoInitializeEx(None, COINIT_APARTMENTTHREADED).is_err() {
            return None;
        }

        let result = (|| -> windows::core::Result<Position> {
            // 创建 UI Automation 实例
            let automation: IUIAutomation =
                CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER)?;

            // 获取焦点元素
            let focused_element = automation.GetFocusedElement()?;

            // 尝试获取 TextPattern
            let text_pattern: IUIAutomationTextPattern =
                focused_element.GetCurrentPatternAs(UIA_TextPatternId)?;

            // 获取选区（光标位置）
            let selection_array = text_pattern.GetSelection()?;
            let length = selection_array.Length()?;

            if length > 0 {
                let selection_range = selection_array.GetElement(0)?;

                // 获取选区的边界矩形（返回 *mut SAFEARRAY）
                let rects = selection_range.GetBoundingRectangles()?;

                let rect_data = std::slice::from_raw_parts(rects as *mut f64, 4);
                let x = rect_data[0] as i32;
                let y = rect_data[1] as i32;
                let height = rect_data[3] as i32;

                return Ok(Position {
                    x,
                    y: y + height, // 光标底部位置
                });
            }

            Err(Error::from(windows::core::HRESULT(-1)))
        })();

        CoUninitialize();

        result.ok()
    }
}
