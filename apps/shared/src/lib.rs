#![no_std]

// #[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
// #[repr(C)]
// pub struct EventData {
//     pub func: [u8; 128],
//     pub query: Option<[u8; 200]>,
//     pub event: Event,
//     pub thread_id: u32,
// }

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum Event {
    Entry(PostgresEntry),
    Return(FunctionName, ThreadId),
}

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum PostgresEntry {
    ExecSimpleQuery(QueryText, ThreadId, Pid),
    ExecParseMessage(QueryText, ThreadId, Pid),
    ExecBindMessage(ThreadId, Pid),
    ExecExecuteMessage(ThreadId, Pid),
    Other(FunctionName, ThreadId),
}

pub const FUNC_NAME_LEN: usize = 50;
pub const QUERY_TEXT_LEN: usize = 228;

pub type FunctionName = [u8; FUNC_NAME_LEN];
pub type QueryText = [u8; QUERY_TEXT_LEN];
pub type ThreadId = u32;
pub type Pid = u32;
