use pgrx::pg_sys;

/// Encode a ctid (block number, offset) into a string document ID.
/// Format: "{block}_{offset}" e.g. "42_3"
pub fn ctid_to_doc_id(ctid: pg_sys::ItemPointerData) -> String {
    let block = unsafe { pg_sys::ItemPointerGetBlockNumberNoCheck(&ctid) };
    let offset = unsafe { pg_sys::ItemPointerGetOffsetNumberNoCheck(&ctid) };
    format!("{}_{}", block, offset)
}

/// Decode a string document ID back into a ctid.
/// Returns None if the string is not in "{block}_{offset}" format.
pub fn doc_id_to_ctid(doc_id: &str) -> Option<pg_sys::ItemPointerData> {
    let (block_str, offset_str) = doc_id.split_once('_')?;
    let block: u32 = block_str.parse().ok()?;
    let offset: u16 = offset_str.parse().ok()?;
    let mut tid: pg_sys::ItemPointerData = unsafe { std::mem::zeroed() };
    unsafe {
        pg_sys::ItemPointerSet(&mut tid, block, offset);
    }
    Some(tid)
}
