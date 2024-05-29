use std::{fs::File, io::Write};

include!("../shared/src/postgres_funcs.rs");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut hooks = File::create("src/generated/hooks.rs")?;
    hooks.write_all(generate_ebpf_hooks(POSTGRES_FUNCS).to_string().as_bytes())?;
    Ok(())
}

fn generate_ebpf_hooks(postgres_funcs: &[&str]) -> String {
    const HOOK: &str = r#"
      #[uprobe]
      pub fn [NAME]_entry(ctx: ProbeContext) -> u32 {
        let thread_id = ctx.tgid();
        let func = str_to_func("[NAME]");
        submit_entry(ctx, PostgresEntry::Other(func, thread_id));
        0
      }

      #[uretprobe]
      pub fn [NAME]_return(ctx: ProbeContext) -> u32 {
        let thread_id = ctx.tgid();
        submit_return(ctx, "[NAME]", thread_id);
        0
      }
    "#;
    let mut hooks = String::new();
    hooks.push_str(
        r#"
        use aya_ebpf::{
            macros::{uprobe, uretprobe},
            programs::ProbeContext,
            EbpfContext,
        };
        use crate::hooks::{submit_entry, submit_return, str_to_func};
        use pg_ferret_shared::{
            PostgresEntry,
        };
        "#,
    );
    for func in postgres_funcs {
        hooks.push_str(&HOOK.replace("[NAME]", func));
    }
    hooks
}
