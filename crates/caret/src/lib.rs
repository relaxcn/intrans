/// 光标位置获取模块
/// 
/// 该模块提供跨平台的光标位置获取功能，根据不同操作系统使用不同的实现策略。

// 平台特定实现
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "linux")]
mod linux;

/// 物理屏幕位置坐标
#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl From<Position> for tauri::PhysicalPosition<i32> {
    fn from(pos: Position) -> Self {
        tauri::PhysicalPosition { x: pos.x, y: pos.y }
    }
}

/// 获取当前活动窗口的光标位置
/// 
/// 返回 `Some(Position)` 如果成功获取，否则返回 `None`
/// 
/// # 平台特定行为
/// 
/// - **Windows**: 使用混合策略（GetGUIThreadInfo + UI Automation）
/// - **macOS**: 待实现
/// - **Linux**: 待实现
pub fn get_position() -> Option<Position> {
    #[cfg(target_os = "windows")]
    {
        windows::get_position()
    }

    #[cfg(target_os = "macos")]
    {
        macos::get_position()
    }

    #[cfg(target_os = "linux")]
    {
        linux::get_position()
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        None
    }
}

/// 初始化光标获取模块
/// 
/// 建议在应用启动时调用此函数，以确保某些需要后台监听的策略（如 Windows MSAA）能正常工作。
pub fn init() {
    #[cfg(target_os = "windows")]
    {
        windows::ensure_msaa_hook_started();
    }
}
