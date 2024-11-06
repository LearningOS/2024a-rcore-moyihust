//! Process management syscalls
use crate::{
    config::{MAX_SYSCALL_NUM, PAGE_SIZE,MAXVA},
    task::{
        change_program_brk, exit_current_and_run_next, suspend_current_and_run_next, TaskStatus,current_user_token,get_current_task_status, 
        get_current_task_syscall_count, get_current_task_start_time,get_current_task_page_table,create_new_map_area,unmap_consecutive_area,
    },
    mm::page_table::translated_byte_buffer,
    timer::{get_time_us,get_time_ms},
    mm::{VPNRange, VirtAddr, VirtPageNum, MapPermission},

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
    let us = get_time_us();
    let dst_vec = translated_byte_buffer(
        current_user_token(),
        _ts as *const u8, core::mem::size_of::<TimeVal>()
    );
    let ref time_val = TimeVal {
            sec: us / 1_000_000,
            usec: us % 1_000_000,
    };
    let src_ptr = time_val as *const TimeVal;
    for (idx, dst) in dst_vec.into_iter().enumerate() {
        let unit_len = dst.len();
        unsafe {
            dst.copy_from_slice(core::slice::from_raw_parts(
                src_ptr.wrapping_byte_add(idx * unit_len) as *const u8,
                unit_len)
            );
        }
    }
    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(_ti: *mut TaskInfo) -> isize {
    // trace!("kernel: sys_task_info NOT IMPLEMENTED YET!");
    let dst_vec = translated_byte_buffer(
        current_user_token(),
        _ti as *const u8, core::mem::size_of::<TaskInfo>()
    );
    let current_time=get_time_ms();
    let ref task_info = TaskInfo {
        status: get_current_task_status(),
        syscall_times: get_current_task_syscall_count(),
        time: current_time-get_current_task_start_time(),
    };
    let src_ptr = task_info as *const TaskInfo;
    for (idx, dst) in dst_vec.into_iter().enumerate() {
        let unit_len = dst.len();
        unsafe {
            dst.copy_from_slice(core::slice::from_raw_parts(
                src_ptr.wrapping_byte_add(idx * unit_len) as *const u8,
                unit_len)
            );
        }
    }
    0
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    // trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    if _start%PAGE_SIZE!=0||
    _port &!0x7!=0||
    _port &0x7==0||
    _start>=MAXVA{
        trace!("kernel: sys_mmap invalid args");
        return -1;
    }
    let start_va:VirtPageNum=VirtAddr::from(_start).floor();
    let end_va:VirtPageNum=VirtAddr::from(_start+_len).ceil();
    let vpns=VPNRange::new(start_va,end_va);
    for vpn in vpns{
        if let Some(pte)=get_current_task_page_table(vpn){
            if pte.is_valid(){
                return -1;
            }
        }
    }
    trace!("kernel: sys_mmap start:{:#x} len:{:#x} port:{:#x}",_start,_len,_port);
    create_new_map_area(
        start_va.into(),
        end_va.into(),
        MapPermission::from_bits_truncate((_port << 1) as u8) | MapPermission::U
    );
    0
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    // trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    if _start>=MAXVA||_start%PAGE_SIZE!=0{
        trace!("kernel: sys_munmap invalid args");
        return -1;
    }

    let mut mlen=_len;
    if _start>MAXVA-mlen{
        mlen=MAXVA-_start;
    }
    println!("kernel: sys_munmap start:{:#x} len:{:#x}",_start,mlen);
    unmap_consecutive_area(_start,mlen)
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
