pub const POSTGRES_TOP_LEVEL_COMMAND_FUNCS: &[&str] = &[
    "exec_simple_query",
    "exec_parse_message",
    "exec_bind_message",
    "exec_execute_message",
];

pub const POSTGRES_FUNCS: &[&str] = &[
    // "exec_simple_query",
    // "exec_parse_message",
    // "exec_bind_message",
    // "exec_execute_message",
    "pg_parse_query",
    "pg_analyze_and_rewrite_varparams",
    "pg_plan_queries",
    "pg_plan_query",
    "planner",
    "BeginImplicitTransactionBlock",
    "PopActiveSnapshot",
    "CreatePortal",
    "PortalDefineQuery",
    "PortalStart",
    "PortalRun",
    "BeginCommand",
    "start_xact_command",
    "finish_xact_command",
    // "TRACE_POSTGRESQL_QUERY_EXECUTE_START",
    // "TRACE_POSTGRESQL_QUERY_EXECUTE_DONE",
    "FillPortalStore",
    "PortalRunSelect",
    "PortalRunMulti",
    "ProcessQuery",
    "ExecutorStart",
    "ExecutorRun",
    "ExecutorFinish",
    "ExecutePlan",
];
