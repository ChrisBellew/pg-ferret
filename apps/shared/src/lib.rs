#![no_std]

/// A message for indicating that a postgres function
/// is starting invocation (entry) or is returning (return).
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum Event {
    Entry(PostgresEntry),
    Return(FunctionName, ThreadId),
}

/// A message for indicating the type of postgres function
/// that is being invoked.
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum PostgresEntry {
    ExecSimpleQuery(QueryText, ThreadId, Pid),
    ExecParseMessage(QueryText, ThreadId, Pid),
    ExecBindMessage(ThreadId, Pid),
    ExecExecuteMessage(ThreadId, Pid),
    Other(FunctionName, ThreadId),
}

/// The length of the function name in postgres.
/// The size must be fixed because we cannot pass
/// variable length arrays from an eBPF program running
/// in the kernel to our collector running in userspace.
/// 50 characters looks large enough to represent most
/// useful function names in the postgres source code.
pub const FUNC_NAME_LEN: usize = 50;

/// The length of the query text in postgres. We'd like
/// this to be as large as possible to capture the full
/// query text, but we are limited by the size of the
/// eBPF stack. The eBPF stack is limited to 512 bytes
/// by default, so we need to keep this value, plus the
/// size of the other fields in the event struct, under
/// 512 bytes.
pub const QUERY_TEXT_LEN: usize = 228;

pub type FunctionName = [u8; FUNC_NAME_LEN];
pub type QueryText = [u8; QUERY_TEXT_LEN];

/// The thread ID is a unique identifier for a thread
/// in the system. It is used to correlate function calls
/// that occur in the same thread in order to reconstruct
/// the order of events and submit them as a trace.
pub type ThreadId = u32;

/// The process ID is a unique identifier for a process
/// in the system. It is used to correlate top level
/// function calls in order to logically group them
/// together in the trace. For example, we can group
/// the three top level function calls exec_parse_message,
/// exec_bind_message, and exec_execute_message together
/// by using the process ID as a key. This triple is
/// common because it represents the three steps of
/// the extended query protocol in postgres.
pub type Pid = u32;
