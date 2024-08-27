use crate::sync::{Condvar, Mutex, MutexBlocking, MutexSpin, Semaphore};
use crate::task::{block_current_and_run_next, current_process, current_task, set_deadlock_detect};
use crate::timer::{add_timer, get_time_ms};
use alloc::sync::Arc;
/// sleep syscall
pub fn sys_sleep(ms: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_sleep",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let expire_ms = get_time_ms() + ms;
    let task = current_task().unwrap();
    add_timer(expire_ms, task);
    block_current_and_run_next();
    0
}
/// mutex create syscall
pub fn sys_mutex_create(blocking: bool) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_mutex_create",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mutex: Option<Arc<dyn Mutex>> = if !blocking {
        Some(Arc::new(MutexSpin::new()))
    } else {
        Some(Arc::new(MutexBlocking::new()))
    };
    let mut process_inner = process.inner_exclusive_access();
    if let Some(id) = process_inner
        .mutex_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.mutex_list[id] = mutex;
        id as isize
    } else {
        process_inner.mutex_list.push(mutex);
        process_inner.mutex_detector.push_resource(1);
        process_inner.mutex_list.len() as isize - 1
    }
}
/// mutex lock syscall
pub fn sys_mutex_lock(mutex_id: usize) -> isize {
    let tid = current_task().unwrap().inner_exclusive_access()
        .res.as_ref().unwrap().tid;
    trace!(
        "kernel:pid[{}] tid[{}] sys_mutex_lock",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
    process_inner.mutex_detector.set_need(tid, mutex_id, 1);
    // process_inner.mutex_detector.alloc_resource(tid, mutex_id, 1);
    if process_inner.mutex_detector.detect() {
        drop(process_inner);
        drop(process);
        mutex.lock();
        let process = current_process();
        let mut process_inner = process.inner_exclusive_access();
        process_inner.mutex_detector.alloc_resource(tid, mutex_id, 1);
        drop(process_inner);
        drop(process);
        0
    }
    else {
        // process_inner.mutex_detector.unalloc_resource(tid, mutex_id, 1);
        // process_inner.mutex_detector.set_need(tid, mutex_id, 1);
        drop(process_inner);
        drop(process);
        -0xDEAD
    }
    
}
/// mutex unlock syscall
pub fn sys_mutex_unlock(mutex_id: usize) -> isize {
    let tid = current_task().unwrap().inner_exclusive_access()
        .res.as_ref().unwrap().tid;
    trace!(
        "kernel:pid[{}] tid[{}] sys_mutex_unlock",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
    drop(process_inner);
    drop(process);
    mutex.unlock();
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    process_inner.mutex_detector.unalloc_resource(tid, mutex_id, 1);
    drop(process_inner);
    drop(process);
    0
}
/// semaphore create syscall
pub fn sys_semaphore_create(res_count: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_semaphore_create",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let id = if let Some(id) = process_inner
        .semaphore_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.semaphore_list[id] = Some(Arc::new(Semaphore::new(res_count)));
        id
    } else {
        process_inner
            .semaphore_list
            .push(Some(Arc::new(Semaphore::new(res_count))));
        process_inner.semaphore_detector.push_resource(res_count as i32);
        process_inner.semaphore_list.len() - 1
    };
    id as isize
}
/// semaphore up syscall
pub fn sys_semaphore_up(sem_id: usize) -> isize {
    let tid = current_task().unwrap().inner_exclusive_access()
        .res.as_ref().unwrap().tid;
    trace!(
        "kernel:pid[{}] tid[{}] sys_semaphore_up",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let sem = Arc::clone(process_inner.semaphore_list[sem_id].as_ref().unwrap());
    /*
    println!("deallocing:");
    for i in process_inner.semaphore_detector.available.iter() {
        println!("Available: {}", i);
    }
    for i in process_inner.semaphore_detector.allocation[tid].iter() {
        println!("Allocation: {}", i);
    }
    for i in process_inner.semaphore_detector.need[tid].iter() {
        println!("Need: {}", i);
    }
    */
    drop(process_inner);
    sem.up();
    let mut process_inner = process.inner_exclusive_access();
    process_inner.semaphore_detector.unalloc_resource(tid, sem_id, 1);
    0
}
/// semaphore down syscall
pub fn sys_semaphore_down(sem_id: usize) -> isize {
    let tid = current_task().unwrap().inner_exclusive_access()
        .res.as_ref().unwrap().tid;
    trace!(
        "kernel:pid[{}] tid[{}] sys_semaphore_down",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let sem = Arc::clone(process_inner.semaphore_list[sem_id].as_ref().unwrap());
    // process_inner.semaphore_detector.alloc_resource(tid, sem_id, 1);
    process_inner.semaphore_detector.set_need(tid, sem_id, 1);
    /*
    println!("detect ans: {}, tid: {}, sem_id: {}", process_inner.semaphore_detector.detect(), tid, sem_id);
    for i in process_inner.semaphore_detector.available.iter() {
        println!("Available: {}", i);
    }
    for i in process_inner.semaphore_detector.allocation[tid].iter() {
        println!("Allocation: {}", i);
    }
    for i in process_inner.semaphore_detector.need[tid].iter() {
        println!("Need: {}", i);
    }
    */
    if process_inner.semaphore_detector.detect() {
        drop(process_inner);
        sem.down();
        let mut process_inner = process.inner_exclusive_access();
        process_inner.semaphore_detector.alloc_resource(tid, sem_id, 1);
        drop(process_inner);
        0
    }
    else {
        // process_inner.semaphore_detector.unalloc_resource(tid, sem_id, 1);
        // process_inner.semaphore_detector.set_need(tid, sem_id, 1);
        drop(process_inner);
        -0xDEAD
    }
    
}
/// condvar create syscall
pub fn sys_condvar_create() -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_condvar_create",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let id = if let Some(id) = process_inner
        .condvar_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.condvar_list[id] = Some(Arc::new(Condvar::new()));
        id
    } else {
        process_inner
            .condvar_list
            .push(Some(Arc::new(Condvar::new())));
        process_inner.condvar_list.len() - 1
    };
    id as isize
}
/// condvar signal syscall
pub fn sys_condvar_signal(condvar_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_condvar_signal",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let condvar = Arc::clone(process_inner.condvar_list[condvar_id].as_ref().unwrap());
    drop(process_inner);
    condvar.signal();
    0
}
/// condvar wait syscall
pub fn sys_condvar_wait(condvar_id: usize, mutex_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_condvar_wait",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let condvar = Arc::clone(process_inner.condvar_list[condvar_id].as_ref().unwrap());
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
    drop(process_inner);
    condvar.wait(mutex);
    0
}
/// enable deadlock detection syscall
///
/// YOUR JOB: Implement deadlock detection, but might not all in this syscall
pub fn sys_enable_deadlock_detect(_enabled: usize) -> isize {
    trace!("kernel: sys_enable_deadlock_detect");
    match _enabled {
        0 => {
            set_deadlock_detect(false);
            return 0;
        },
        1 => {
            set_deadlock_detect(true);
            return 0;
        },
        _ => { return -1; }
    }
}
