use crate::errors::*;
use crate::utils::*;
use crate::wikilink::resolve_wikilink;

pub fn resolve(wikilink: &str) -> Result<()> {
    let vault_root = find_vault_root()?;
    
    match resolve_wikilink(&vault_root, wikilink) {
        Ok(path) => {
            println!("{}", path.display());
            Ok(())
        }
        Err(e) => Err(e),
    }
}
