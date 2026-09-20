//! Make a facts-file copy a store `acq` can open, for the build plan's M3
//! and M4 (`search/BUILD-PLAN.md`, "Measurements the build owes"): given a
//! sqlite `.backup` copy and an empty directory, record the copy's own
//! account in an index there and move the copy to the path that index
//! names, under the mock provider's directory — so that
//! `ACQ_PROVIDER=mock ACQ_STORE_DIR=<dir> acq search …` reads the copy and
//! can never reach the owner's store (the plan's rule 7). Never in the
//! gate: the input is a track's `raw/`.

use acquisition_store::corpus::RealmScope;
use acquisition_store::{Index, Store, account_path};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let (Some(copy), Some(dir)) = (args.next(), args.next()) else {
        return Err("usage: copy-as-store <copy.db> <store-dir>".into());
    };
    let (uuid, name) = Store::open(copy.as_ref())?.read_corpus(RealmScope::All, |header, _| {
        Ok((header.account_uuid.clone(), header.account_name.clone()))
    })?;
    let name = name.unwrap_or_else(|| uuid.clone());
    let mock = std::path::Path::new(&dir).join("mock");
    std::fs::create_dir_all(&mock)?;
    Index::load(&mock)?.record_login(&name, &uuid, false, acquisition_store::now())?;
    let to = account_path(&mock, &name);
    std::fs::rename(&copy, &to)?;
    println!("{}", to.display());
    Ok(())
}
