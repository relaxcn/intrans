# Caret - 跨平台光标位置检测库

一个用于检测当前活动窗口光标位置的 Rust 库，支持多种操作系统。

## 功能特性

- 🎯 **跨平台支持**: Windows、macOS、Linux
- ⚡ **多策略检测**: 针对不同应用使用最优策略
- 🔧 **易于集成**: 简洁的 API，开箱即用

## 平台支持

### Windows

使用混合策略，自动适配不同类型的应用：

1. **GetGUIThreadInfo** - 适用于标准 Windows 控件
   - Sublime Text
   - Notepad
   - Visual Studio
   - 其他使用标准控件的应用

2. **UI Automation** - 适用于自定义渲染的现代应用
   - Chrome / Edge
   - VS Code
   - Electron 应用
   - 其他基于 Chromium 的应用

### macOS

⚠️ 待实现

### Linux

⚠️ 待实现

## 使用示例

```rust
use caret;

fn main() {
    match caret::get_position() {
        Some(pos) => {
            println!("光标位置: x={}, y={}", pos.x, pos.y);
        }
        None => {
            println!("无法获取光标位置");
        }
    }
}
```

## API 文档

### `Position`

表示屏幕上的物理坐标：

```rust
pub struct Position {
    pub x: i32,
    pub y: i32,
}
```

### `get_position()`

获取当前活动窗口的光标位置：

```rust
pub fn get_position() -> Option<Position>
```

- **返回值**: 
  - `Some(Position)` - 成功获取光标位置
  - `None` - 无法获取（不支持的应用或平台）

## 工作原理

### Windows 平台

1. 首先尝试 `GetGUIThreadInfo` API（快速）
2. 如果失败，回退到 UI Automation（兼容性好）
3. 返回光标底部位置坐标

### 为什么在 Chrome 中需要 UI Automation？

Chrome 使用自定义渲染引擎（Blink），不依赖 Windows 标准控件，因此：

- ❌ `GetGUIThreadInfo` 无法获取光标信息
- ✅ UI Automation 可以通过辅助功能接口获取

这就像输入法的工作原理：应用通过 TSF/UI Automation 主动告知光标位置。

## 依赖

### Windows

- `windows` crate (0.62.2)
  - Win32_Foundation
  - Win32_Graphics_Gdi
  - Win32_System_Com
  - Win32_UI_Accessibility
  - Win32_UI_WindowsAndMessaging

### macOS

暂无

### Linux

暂无

## 许可证

MIT
