//! Bounded native monitor inventory, in Windows callback order.
//!
//! Indices deliberately match winit 0.30.13's Windows `available_monitors`:
//! both enumerate all monitors with EnumDisplayMonitors(None, None, ...),
//! without sorting or filtering. Indices are snapshot-local, not persistent
//! identities. Revalidate `id` when using a saved selection after hotplug.
//! This module never changes a window, display mode, DPI policy or routing.

pub const MAX_DISPLAYS: usize = 32;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayInfo {
    pub index: usize,
    /// Windows display-device name (szDevice), not a human display label.
    pub id: String,
    pub primary: bool,
    /// Win32 virtual-desktop coordinates; query on the GUI thread after winit
    /// initializes DPI awareness so these match its physical monitor bounds.
    pub position: [i32; 2],
    pub size: [u32; 2],
}

#[derive(Debug, thiserror::Error)]
pub enum DisplayError {
    #[error("Native display enumeration is supported on Windows only")]
    Unsupported,
    #[error("{operation} failed (Windows error {code})")]
    Native { operation: &'static str, code: u32 },
    #[error("Display inventory exceeds the bounded 32-monitor limit")]
    Capacity,
    #[error("Invalid display inventory: {0}")]
    InvalidInventory(&'static str),
    #[error(
        "The operator window is invalid, off all displays, or its display changed; refresh displays"
    )]
    MissingWindowMonitor,
}
pub type Result<T> = std::result::Result<T, DisplayError>;

/// Enumerate all monitors in the exact native callback order, or fail entirely.
/// There is no primary-display, nearest-display or partial-inventory fallback.
pub fn enumerate() -> Result<Vec<DisplayInfo>> {
    #[cfg(windows)]
    {
        let inventory = native::enumerate_records()?;
        convert_records(inventory.records()?)
    }
    #[cfg(not(windows))]
    Err(DisplayError::Unsupported)
}

/// Resolve the actual operator window's display without a primary/nearest
/// fallback. `hwnd` is an opaque Win32 HWND value; no Rust pointer is read from
/// it. The caller should pass its own live root-window handle, not persist it.
/// The returned index is valid only for the current monitor topology.
pub fn operator_display_index(hwnd: usize) -> Result<usize> {
    #[cfg(windows)]
    {
        let monitor = native::operator_monitor(hwnd)?;
        let inventory = native::enumerate_records()?;
        let records = inventory.records()?;
        // Validate the complete snapshot, rather than returning an index into
        // an inventory that enumerate() would reject.
        convert_records(records)?;
        index_for_handle(records, monitor)
    }
    #[cfg(not(windows))]
    {
        let _ = hwnd;
        Err(DisplayError::Unsupported)
    }
}

#[cfg(any(windows, test))]
#[derive(Clone, Copy)]
struct Record {
    handle: usize,
    device: [u16; 32],
    rect: [i32; 4],
    primary: bool,
}
#[cfg(windows)]
impl Record {
    const EMPTY: Self = Self {
        handle: 0,
        device: [0; 32],
        rect: [0; 4],
        primary: false,
    };
}

#[cfg(any(windows, test))]
fn convert_records(records: &[Record]) -> Result<Vec<DisplayInfo>> {
    if records.is_empty() {
        return Err(DisplayError::InvalidInventory("no monitors are connected"));
    }
    if records.len() > MAX_DISPLAYS {
        return Err(DisplayError::Capacity);
    }
    let mut output: Vec<DisplayInfo> = Vec::new();
    output
        .try_reserve_exact(records.len())
        .map_err(|_| DisplayError::InvalidInventory("monitor allocation failed"))?;
    let mut primary_count = 0_usize;
    for (index, record) in records.iter().enumerate() {
        if record.handle == 0
            || records
                .iter()
                .take(index)
                .any(|previous| previous.handle == record.handle)
        {
            return Err(DisplayError::InvalidInventory(
                "monitor handles must be nonzero and unique",
            ));
        }
        let [left, top, right, bottom] = record.rect;
        let width = right
            .checked_sub(left)
            .and_then(|value| u32::try_from(value).ok())
            .filter(|value| *value > 0);
        let height = bottom
            .checked_sub(top)
            .and_then(|value| u32::try_from(value).ok())
            .filter(|value| *value > 0);
        let (Some(width), Some(height)) = (width, height) else {
            return Err(DisplayError::InvalidInventory(
                "monitor rectangle is empty or overflows",
            ));
        };
        let terminator = record.device.iter().position(|value| *value == 0).ok_or(
            DisplayError::InvalidInventory("device identity is not terminated"),
        )?;
        let device = record
            .device
            .get(..terminator)
            .ok_or(DisplayError::InvalidInventory(
                "device identity length is invalid",
            ))?;
        let id = String::from_utf16(device)
            .map_err(|_| DisplayError::InvalidInventory("device identity is not valid UTF-16"))?;
        if id.trim().is_empty()
            || id.chars().any(char::is_control)
            || output.iter().any(|previous| previous.id == id)
        {
            return Err(DisplayError::InvalidInventory(
                "device identities must be nonempty, printable and unique",
            ));
        }
        if record.primary {
            primary_count = primary_count
                .checked_add(1)
                .ok_or(DisplayError::InvalidInventory(
                    "primary monitor count overflows",
                ))?;
        }
        output.push(DisplayInfo {
            index,
            id,
            primary: record.primary,
            position: [left, top],
            size: [width, height],
        });
    }
    if primary_count != 1 {
        return Err(DisplayError::InvalidInventory(
            "exactly one primary monitor is required",
        ));
    }
    Ok(output)
}

#[cfg(any(windows, test))]
fn index_for_handle(records: &[Record], monitor: usize) -> Result<usize> {
    if monitor == 0 {
        return Err(DisplayError::MissingWindowMonitor);
    }
    records
        .iter()
        .position(|record| record.handle == monitor)
        .ok_or(DisplayError::MissingWindowMonitor)
}

#[cfg(windows)]
mod native {
    use super::*;
    use windows::{
        Win32::{
            Foundation::{GetLastError, HWND, LPARAM, RECT},
            Graphics::Gdi::{
                EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITOR_DEFAULTTONULL,
                MONITORINFO, MONITORINFOEXW, MonitorFromWindow,
            },
        },
        core::BOOL,
    };

    // MONITORINFO.dwFlags bit 0 is MONITORINFOF_PRIMARY, per Win32 MONITORINFO.
    // Keeping the documented value here avoids unrelated Windows UI features.
    const PRIMARY_FLAG: u32 = 1;
    #[derive(Clone, Copy)]
    enum Failure {
        None,
        Capacity,
        MonitorInfo(u32),
    }
    pub(super) struct Inventory {
        slots: [Record; MAX_DISPLAYS],
        length: usize,
        info_size: u32,
        failure: Failure,
    }
    impl Inventory {
        pub(super) fn records(&self) -> Result<&[Record]> {
            self.slots.get(..self.length).ok_or(DisplayError::Capacity)
        }
    }

    /// Synchronous Windows callback: only fixed-size stack/local copies and
    /// checked slot access. No allocation, user callbacks, locks or panics.
    unsafe extern "system" fn collect(
        monitor: HMONITOR,
        _: HDC,
        _: *mut RECT,
        data: LPARAM,
    ) -> BOOL {
        // SAFETY: enumerate_records passes an exclusive pointer to its live
        // stack Inventory in LPARAM. EnumDisplayMonitors invokes callbacks
        // synchronously on that thread, preserving the value and not retaining
        // it; no other reference accesses Inventory until the call returns.
        // The null check handles an absent callback context without dereference.
        let Some(inventory) = (unsafe { (data.0 as *mut Inventory).as_mut() }) else {
            return BOOL(0);
        };
        let Some(next_length) = inventory
            .length
            .checked_add(1)
            .filter(|length| *length <= MAX_DISPLAYS)
        else {
            inventory.failure = Failure::Capacity;
            return BOOL(0);
        };
        let mut info = MONITORINFOEXW::default();
        info.monitorInfo.cbSize = inventory.info_size;
        // SAFETY: monitor is the current HMONITOR supplied by the synchronous
        // Windows enumeration. info is exclusively owned, correctly aligned,
        // initialized and live throughout this callback. Its repr(C) prefix is
        // MONITORINFO and cbSize is the checked full MONITORINFOEXW byte size.
        // Windows writes at most that full buffer and retains no pointer.
        let succeeded = unsafe {
            GetMonitorInfoW(
                monitor,
                (&mut info as *mut MONITORINFOEXW).cast::<MONITORINFO>(),
            )
        };
        if !succeeded.as_bool() {
            // SAFETY: GetLastError reads this callback thread's last-error value
            // immediately after the failed API; it takes no pointers or buffers.
            inventory.failure = Failure::MonitorInfo(unsafe { GetLastError() }.0);
            return BOOL(0);
        }
        let Some(slot) = inventory.slots.get_mut(inventory.length) else {
            inventory.failure = Failure::Capacity;
            return BOOL(0);
        };
        *slot = Record {
            handle: monitor.0 as usize,
            device: info.szDevice,
            rect: [
                info.monitorInfo.rcMonitor.left,
                info.monitorInfo.rcMonitor.top,
                info.monitorInfo.rcMonitor.right,
                info.monitorInfo.rcMonitor.bottom,
            ],
            primary: info.monitorInfo.dwFlags & PRIMARY_FLAG != 0,
        };
        inventory.length = next_length;
        BOOL(1)
    }

    pub(super) fn enumerate_records() -> Result<Inventory> {
        let info_size = u32::try_from(std::mem::size_of::<MONITORINFOEXW>())
            .map_err(|_| DisplayError::InvalidInventory("native monitor structure is too large"))?;
        let mut inventory = Inventory {
            slots: [Record::EMPTY; MAX_DISPLAYS],
            length: 0,
            info_size,
            failure: Failure::None,
        };
        // SAFETY: no HDC or clip rectangle requests every monitor, matching
        // winit's Windows inventory. collect has the exact system ABI signature.
        // LPARAM points to an exclusive, aligned stack Inventory whose fixed
        // 32-record buffer outlives every synchronous callback on this thread.
        // Neither Windows nor collect retains the pointer after enumeration.
        let succeeded = unsafe {
            EnumDisplayMonitors(
                None,
                None,
                Some(collect),
                LPARAM((&mut inventory as *mut Inventory) as isize),
            )
        };
        match inventory.failure {
            Failure::Capacity => Err(DisplayError::Capacity),
            Failure::MonitorInfo(code) => Err(DisplayError::Native {
                operation: "GetMonitorInfoW",
                code,
            }),
            Failure::None if !succeeded.as_bool() => {
                // SAFETY: reads only this thread's last-error slot immediately
                // after EnumDisplayMonitors failed; no pointers are involved.
                let code = unsafe { GetLastError() }.0;
                Err(DisplayError::Native {
                    operation: "EnumDisplayMonitors",
                    code,
                })
            }
            Failure::None => Ok(inventory),
        }
    }

    pub(super) fn operator_monitor(hwnd: usize) -> Result<usize> {
        if hwnd == 0 {
            return Err(DisplayError::MissingWindowMonitor);
        }
        // SAFETY: HWND is an opaque OS handle, never dereferenced by Rust. The
        // Windows API validates it and returns null for an invalid/offscreen
        // window with MONITOR_DEFAULTTONULL. No borrowed memory is passed or
        // retained; the synchronous call occurs on the caller's window thread.
        let monitor = unsafe {
            MonitorFromWindow(HWND(hwnd as *mut std::ffi::c_void), MONITOR_DEFAULTTONULL)
        };
        if monitor.0.is_null() {
            Err(DisplayError::MissingWindowMonitor)
        } else {
            Ok(monitor.0 as usize)
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::indexing_slicing)]
    use super::*;
    fn record(handle: usize, name: &str, primary: bool, rect: [i32; 4]) -> Record {
        let mut device = [0; 32];
        for (slot, value) in device.iter_mut().zip(name.encode_utf16()) {
            *slot = value;
        }
        Record {
            handle,
            device,
            primary,
            rect,
        }
    }
    #[test]
    fn native_order_survives_primary_and_negative_desktop_coordinates() {
        let records = [
            record(7, "DISPLAY3", false, [-1920, -100, 0, 980]),
            record(4, "DISPLAY1", true, [0, 0, 2560, 1440]),
            record(9, "DISPLAY2", false, [2560, 0, 4480, 1080]),
        ];
        let output = convert_records(&records).unwrap();
        assert_eq!(
            output
                .iter()
                .map(|display| display.id.as_str())
                .collect::<Vec<_>>(),
            vec!["DISPLAY3", "DISPLAY1", "DISPLAY2"]
        );
        assert_eq!(output[0].index, 0);
        assert_eq!(output[0].position, [-1920, -100]);
        assert_eq!(output[0].size, [1920, 1080]);
        assert!(output[1].primary);
        assert_eq!(index_for_handle(&records, 4).unwrap(), 1);
        assert_eq!(index_for_handle(&records, 7).unwrap(), 0);
        assert!(index_for_handle(&records, 99).is_err()); // no primary fallback
        assert!(index_for_handle(&records, 0).is_err());
    }
    #[test]
    fn malformed_records_never_return_a_partial_or_renumbered_inventory() {
        let valid = record(1, "DISPLAY1", true, [0, 0, 1920, 1080]);
        assert!(convert_records(&[]).is_err());
        assert!(convert_records(&[valid; MAX_DISPLAYS + 1]).is_err());
        for bad in [
            record(2, "DISPLAY2", false, [0, 0, 0, 1080]),
            record(2, "DISPLAY2", false, [i32::MIN, 0, i32::MAX, 1080]),
            record(2, "", false, [0, 0, 10, 10]),
            record(2, "DISPLAY\n2", false, [0, 0, 10, 10]),
            record(1, "DISPLAY2", false, [0, 0, 10, 10]),
            record(0, "DISPLAY2", false, [0, 0, 10, 10]),
            record(2, "DISPLAY1", false, [0, 0, 10, 10]),
            record(2, "DISPLAY2", true, [0, 0, 10, 10]),
        ] {
            assert!(convert_records(&[valid, bad]).is_err());
        }
        let mut invalid_utf16 = record(2, "DISPLAY2", false, [0, 0, 10, 10]);
        invalid_utf16.device = [0; 32];
        invalid_utf16.device[0] = 0xD800;
        assert!(convert_records(&[valid, invalid_utf16]).is_err());
        let mut unterminated = valid;
        unterminated.device = [65; 32];
        assert!(convert_records(&[unterminated]).is_err());
        let mut no_primary = valid;
        no_primary.primary = false;
        assert!(convert_records(&[no_primary]).is_err());
    }
    #[cfg(not(windows))]
    #[test]
    fn unsupported_platform_never_invents_a_display() {
        assert!(matches!(enumerate(), Err(DisplayError::Unsupported)));
        assert!(matches!(
            operator_display_index(1),
            Err(DisplayError::Unsupported)
        ));
    }
}
