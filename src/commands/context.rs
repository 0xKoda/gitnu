use crate::errors::*;
use crate::storage::Storage;
use crate::context::ContextManager;
use crate::utils::*;
use colored::Colorize;

pub fn context(clipboard: bool, json: bool, compress: bool) -> Result<()> {
    let vault_root = find_vault_root()?;
    let storage = Storage::new(vault_root.clone());
    let context_mgr = ContextManager::new(storage);

    let content = context_mgr.load_context(compress)?;

    if json {
        // Output as structured JSON
        let files = context_mgr.get_all_files()?;
        let json_output = serde_json::json!({
            "files": files,
            "content": content,
            "token_estimate": estimate_tokens(&content),
        });
        println!("{}", serde_json::to_string_pretty(&json_output)?);
    } else if clipboard {
        // Copy to clipboard (placeholder - would need clipboard crate)
        println!("{}", "Clipboard support not yet implemented".yellow());
        println!("{}", "Context output:".bold());
        println!("{}", content);
    } else {
        // Output to stdout
        println!("{}", content);
    }

    Ok(())
}
