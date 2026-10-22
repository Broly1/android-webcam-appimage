// The few v4l2-ctl queries the app needs on /dev/video9, done with ioctls directly so the
// AppImage doesn't depend on v4l-utils (not installed by default on Ubuntu).

use std::fs::{File, OpenOptions};
use std::os::fd::AsRawFd;

const DEVICE: &str = "/dev/video9";

// _IOWR('V', nr, size) from linux/videodev2.h (x86_64).
const VIDIOC_G_FMT: libc::c_ulong = 0xC0D0_5604; // struct v4l2_format, 208 bytes
const VIDIOC_G_CTRL: libc::c_ulong = 0xC008_561B; // struct v4l2_control
const VIDIOC_S_CTRL: libc::c_ulong = 0xC008_561C;

const V4L2_BUF_TYPE_VIDEO_CAPTURE: u32 = 1;
/// v4l2loopback's keep_format control (V4L2_CID_USER_BASE | 0xf000).
const CID_KEEP_FORMAT: u32 = 0x0098_f900;

#[repr(C)]
struct Control {
    id: u32,
    value: i32,
}

/// struct v4l2_format: the buffer type, then a union aligned to 8 bytes whose first member,
/// struct v4l2_pix_format, starts with width and height.
#[repr(C, align(8))]
struct Format {
    kind: u32,
    _pad: u32,
    width: u32,
    height: u32,
    _rest: [u8; 192],
}

fn open() -> Option<File> {
    OpenOptions::new().read(true).write(true).open(DEVICE).ok()
}

/// `v4l2-ctl --get-fmt-video`: the capture size, or None if the device has no format yet.
pub fn capture_size() -> Option<(u32, u32)> {
    let dev = open()?;
    let mut fmt = Format { kind: V4L2_BUF_TYPE_VIDEO_CAPTURE, _pad: 0, width: 0, height: 0, _rest: [0; 192] };
    let ret = unsafe { libc::ioctl(dev.as_raw_fd(), VIDIOC_G_FMT, &mut fmt as *mut Format) };
    (ret == 0 && fmt.width > 0 && fmt.height > 0).then_some((fmt.width, fmt.height))
}

/// `v4l2-ctl --get-ctrl keep_format`. Also tells whether the device is usable yet.
pub fn keep_format() -> Option<bool> {
    let dev = open()?;
    let mut ctrl = Control { id: CID_KEEP_FORMAT, value: 0 };
    let ret = unsafe { libc::ioctl(dev.as_raw_fd(), VIDIOC_G_CTRL, &mut ctrl as *mut Control) };
    (ret == 0).then_some(ctrl.value != 0)
}

/// `v4l2-ctl -c keep_format=0|1`.
pub fn set_keep_format(on: bool) -> bool {
    let Some(dev) = open() else { return false };
    let mut ctrl = Control { id: CID_KEEP_FORMAT, value: i32::from(on) };
    unsafe { libc::ioctl(dev.as_raw_fd(), VIDIOC_S_CTRL, &mut ctrl as *mut Control) == 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_layout() {
        // VIDIOC_G_FMT encodes the struct size; it must match the kernel's.
        assert_eq!(std::mem::size_of::<Format>(), 208);
        assert_eq!(std::mem::size_of::<Control>(), 8);
    }
}
