//! Process management syscalls
use alloc::sync::Arc;

use crate::{
    config::{MAX_SYSCALL_NUM, PAGE_SIZE,MAXVA},
    task::{
        change_program_brk, exit_current_and_run_next, suspend_current_and_run_next, TaskStatus,current_user_token,get_current_task_status, 
        get_current_task_syscall_count, get_current_task_start_time,get_current_task_page_table,create_new_map_area,unmap_consecutive_area,
        add_task, current_task, current_user_token,
    },
    mm::page_table::translated_byte_buffer,
    timer::{get_time_us,get_time_ms},
    mm::{VPNRange, VirtAddr, VirtPageNum, MapPermission},
    loader::get_app_data_by_name,
    mm::{translated_refmut, translated_str},
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
pub fn sys_exit(exit_code: i32) -> ! {
    trace!("kernel:pid[{}] sys_exit", current_task().unwrap().pid.0);
    exit_current_and_run_next(exit_code);
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel:pid[{}] sys_yield", current_task().unwrap().pid.0);
    suspend_current_and_run_next();
    0
}

pub fn sys_getpid() -> isize {
    trace!("kernel: sys_getpid pid:{}", current_task().unwrap().pid.0);
    current_task().unwrap().pid.0 as isize
}

pub fn sys_fork() -> isize {
    trace!("kernel:pid[{}] sys_fork", current_task().unwrap().pid.0);
    let current_task = current_task().unwrap();
    let new_task = current_task.fork();
    let new_pid = new_task.pid.0;
    // modify trap context of new_task, because it returns immediately after switching
    let trap_cx = new_task.inner_exclusive_access().get_trap_cx();
    // we do not have to move to next instruction since we have done it before
    // for child process, fork returns 0
    trap_cx.x[10] = 0;
    // add new task to scheduler
    add_task(new_task);
    new_pid as isize
}

pub fn sys_exec(path: *const u8) -> isize {
    trace!("kernel:pid[{}] sys_exec", current_task().unwrap().pid.0);
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(data) = get_app_data_by_name(path.as_str()) {
        let task = current_task().unwrap();
        task.exec(data);
        0
    } else {
        -1
    }
}

/// If there is not a child process whose pid is same as given, return -1.
/// Else if there is a child process but it is still running, return -2.
pub fn sys_waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
    trace!("kernel::pid[{}] sys_waitpid [{}]", current_task().unwrap().pid.0, pid);
    let task = current_task().unwrap();
    // find a child process

    // ---- access current PCB exclusively
    let mut inner = task.inner_exclusive_access();
    if !inner
        .children
        .iter()
        .any(|p| pid == -1 || pid as usize == p.getpid())
    {
        return -1;
        // ---- release current PCB
    }
    let pair = inner.children.iter().enumerate().find(|(_, p)| {
        // ++++ temporarily access child PCB exclusively
        p.inner_exclusive_access().is_zombie() && (pid == -1 || pid as usize == p.getpid())
        // ++++ release child PCB
    });
    if let Some((idx, _)) = pair {
        let child = inner.children.remove(idx);
        // confirm that child will be deallocated after being removed from children list
        assert_eq!(Arc::strong_count(&child), 1);
        let found_pid = child.getpid();
        // ++++ temporarily access child PCB exclusively
        let exit_code = child.inner_exclusive_access().exit_code;
        // ++++ release child PCB
        *translated_refmut(inner.memory_set.token(), exit_code_ptr) = exit_code;
        found_pid as isize
    } else {
        -2
    }
    // ---- release current PCB automatically
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

/// YOUR JOB: Implement mmap.
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

/// YOUR JOB: Implement munmap.
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
    trace!("kernel:pid[{}] sys_sbrk", current_task().unwrap().pid.0);
    if let Some(old_brk) = current_task().unwrap().change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}

/// YOUR JOB: Implement spawn.
/// HINT: fork + exec =/= spawn
pub fn sys_spawn(_path: *const u8) -> isize {
    trace!(
        "kernel:pid[{}] sys_spawn NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );
    -1
}

// YOUR JOB: Set task priority.
pub fn sys_set_priority(_prio: isize) -> isize {
    trace!(
        "kernel:pid[{}] sys_set_priority NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );
    -1
}
