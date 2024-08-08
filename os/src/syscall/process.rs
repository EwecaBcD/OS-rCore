//! Process management syscalls
use crate::{
    config::{MAX_SYSCALL_NUM, PAGE_SIZE}, mm::{translated_mut, MapPermission, VirtAddr}, task::{
        add_map_area, change_program_brk, current_user_token, exit_current_and_run_next, get_current_first_time, get_current_syscall_times, suspend_current_and_run_next, unmap_vp, TaskStatus
    }, timer::{get_time_ms, get_time_us}
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// Task information
#[allow(dead_code)]
pub struct TaskInfo {
    /// Task status in it's life cycle
    status: TaskStatus,
    /// The numbers of syscall called by task
    syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
    time: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let ts = translated_mut(current_user_token(), _ts);
    let us = get_time_us();
    *ts = TimeVal {
        sec: us / 1_000_000,
        usec: us % 1_000_000,
    };
    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(_ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info NOT IMPLEMENTED YET!");
    let ti = translated_mut(current_user_token(), _ti);
    let ms = get_time_ms();
    *ti = TaskInfo {
        status: TaskStatus::Running,
        syscall_times: get_current_syscall_times(),
        time: ms - get_current_first_time()
    };
    0
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    // 检查首地址是否对齐
    if _start % PAGE_SIZE != 0 { return -1; }
    let start_va = VirtAddr::from(_start);
    let page_cnt = (_len - 1 + PAGE_SIZE) / PAGE_SIZE;
    let end_va = VirtAddr::from(start_va.0 + page_cnt * PAGE_SIZE);
    let permission: MapPermission;
    match _port {
        0x0 => return -1,
        0x1 => permission = MapPermission::R | MapPermission::U,
        0x2 => permission = MapPermission::W | MapPermission::U,
        0x3 => permission = MapPermission::R | MapPermission::U | MapPermission::W,
        0x4 => permission = MapPermission::X | MapPermission::U,
        0x5 => permission = MapPermission::R | MapPermission::U | MapPermission::X,
        0x6 => permission = MapPermission::W | MapPermission::U | MapPermission::X,
        0x7 => permission = MapPermission::R | MapPermission::U | MapPermission::X | MapPermission::W,
        _ => return -1
    }
    match add_map_area(start_va, end_va, permission) {
        Ok(_) => 0,
        Err(_) => -1
    }
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    // 检查首地址是否对齐
    if _start % PAGE_SIZE != 0 { return -1; }
    let mut start_vpn = VirtAddr::from(_start).floor();
    let page_cnt = (_len - 1 + PAGE_SIZE) / PAGE_SIZE;
    // let end_va = VirtAddr::from(start_va.0 + page_cnt * PAGE_SIZE);
    for _ in 0..page_cnt {
        match unmap_vp(start_vpn) {
            Ok(_) => { },
            Err(_) => return -1
        }
        start_vpn.0 += 1;
    }
    0
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
