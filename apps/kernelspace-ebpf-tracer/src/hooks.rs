use aya_ebpf::{
    helpers::bpf_probe_read_user_str_bytes,
    macros::{map, uprobe, uretprobe},
    maps::PerfEventArray,
    programs::ProbeContext,
    EbpfContext,
};
use aya_log_ebpf::info;
use core::ffi::c_char;
use pg_ferret_shared::{
    Event, FunctionName, PostgresEntry, QueryText, ThreadId, FUNC_NAME_LEN, QUERY_TEXT_LEN,
};

#[map]
pub static mut EVENTS: PerfEventArray<Event> = PerfEventArray::with_max_entries(1024, 0);

#[uprobe]
pub fn exec_simple_query_entry(ctx: ProbeContext) -> u32 {
    let query = query_text(&ctx);
    let thread_id = ctx.tgid();
    let pid = ctx.pid();
    let event = PostgresEntry::ExecSimpleQuery(query, thread_id, pid);
    submit_entry(ctx, event)
}

#[uretprobe]
pub fn exec_simple_query_return(ctx: ProbeContext) -> u32 {
    let thread_id = ctx.tgid();
    submit_return(ctx, "exec_simple_query", thread_id)
}

#[uprobe]
pub fn exec_parse_message_entry(ctx: ProbeContext) -> u32 {
    let query = query_text(&ctx);
    let thread_id = ctx.tgid();
    let pid = ctx.pid();
    let event = PostgresEntry::ExecParseMessage(query, thread_id, pid);
    submit_entry(ctx, event)
}

#[uretprobe]
pub fn exec_parse_message_return(ctx: ProbeContext) -> u32 {
    let thread_id = ctx.tgid();
    submit_return(ctx, "exec_parse_message", thread_id)
}

#[uprobe]
pub fn exec_bind_message_entry(ctx: ProbeContext) -> u32 {
    let thread_id = ctx.tgid();
    let pid = ctx.pid();
    let event = PostgresEntry::ExecBindMessage(thread_id, pid);
    submit_entry(ctx, event)
}

#[uretprobe]
pub fn exec_bind_message_return(ctx: ProbeContext) -> u32 {
    let thread_id = ctx.tgid();
    submit_return(ctx, "exec_bind_message", thread_id)
}

#[uprobe]
pub fn exec_execute_message_entry(ctx: ProbeContext) -> u32 {
    let thread_id = ctx.tgid();
    let pid = ctx.pid();
    let event = PostgresEntry::ExecExecuteMessage(thread_id, pid);
    submit_entry(ctx, event)
}

#[uretprobe]
pub fn exec_execute_message_return(ctx: ProbeContext) -> u32 {
    let thread_id = ctx.tgid();
    submit_return(ctx, "exec_execute_message", thread_id)
}

fn query_text(ctx: &ProbeContext) -> QueryText {
    let arg0: *const c_char = ctx.arg(0).unwrap();
    let mut buf = [0u8; QUERY_TEXT_LEN];
    unsafe { bpf_probe_read_user_str_bytes(arg0 as *const u8, &mut buf).unwrap() };
    buf
}

pub fn submit_entry(ctx: ProbeContext, event: PostgresEntry) -> u32 {
    unsafe { EVENTS.output(&ctx, &Event::Entry(event), 0) }
    0
}

pub fn submit_return(ctx: ProbeContext, func: &str, thread_id: ThreadId) -> u32 {
    info!(
        &ctx,
        "function {}_return called by /usr/local/pgsql/bin/postgres", func
    );
    let func = str_to_func(func);
    unsafe { EVENTS.output(&ctx, &Event::Return(func, thread_id), 0) }
    0
}

pub fn str_to_func(str: &str) -> FunctionName {
    let mut func_name_bytes = [0u8; FUNC_NAME_LEN];
    let func_bytes = str.as_bytes();
    let len = if func_bytes.len() < FUNC_NAME_LEN {
        func_bytes.len()
    } else {
        FUNC_NAME_LEN
    };
    func_name_bytes[..len].copy_from_slice(&func_bytes[..len]);
    func_name_bytes
}

// pub fn entry(ctx: ProbeContext, func: &str) -> Result<u32, u32> {
//     let thread_id = ctx.tgid();
//     unsafe {
//         EVENTS.output(
//             &ctx,
//             &EventData {
//                 func: convert_func_name(func),
//                 event: Event::Entry,
//                 thread_id,
//             },
//             0,
//         );
//     }
//     info!(
//         &ctx,
//         "function {}_entry called by /usr/local/pgsql/bin/postgres", func
//     );
//     Ok(0)
// }

fn convert_func_name(func: &str) -> [u8; FUNC_NAME_LEN] {
    let mut func_name_bytes = [0u8; FUNC_NAME_LEN];
    let func_bytes = func.as_bytes();
    let len = if func_bytes.len() < FUNC_NAME_LEN {
        func_bytes.len()
    } else {
        FUNC_NAME_LEN
    };
    func_name_bytes[..len].copy_from_slice(&func_bytes[..len]);
    func_name_bytes
}
