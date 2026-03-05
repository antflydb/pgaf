use pgrx::pg_sys;

use super::ctid::ctid_to_doc_id;
use super::options;
use crate::client::AntflyClient;

struct BuildState {
    client: AntflyClient,
    collection: String,
    count: f64,
}

#[pgrx::pg_guard]
pub unsafe extern "C-unwind" fn ambuild(
    heap_relation: pg_sys::Relation,
    index_relation: pg_sys::Relation,
    index_info: *mut pg_sys::IndexInfo,
) -> *mut pg_sys::IndexBuildResult {
    let (url, collection) = unsafe { options::get_options(index_relation) };

    let client = AntflyClient::new(&url).unwrap_or_else(|e| {
        pgrx::error!("pgaf: failed to create Antfly client: {}", e);
    });

    let mut state = BuildState {
        client,
        collection,
        count: 0.0,
    };

    let reltuples = unsafe {
        pg_sys::table_index_build_scan(
            heap_relation,
            index_relation,
            index_info,
            true,
            false,
            Some(build_callback),
            &mut state as *mut BuildState as *mut std::os::raw::c_void,
            std::ptr::null_mut(),
        )
    };

    let result = unsafe {
        pg_sys::palloc0(std::mem::size_of::<pg_sys::IndexBuildResult>())
            as *mut pg_sys::IndexBuildResult
    };
    unsafe {
        (*result).heap_tuples = reltuples;
        (*result).index_tuples = state.count;
    }

    result
}

#[pgrx::pg_guard]
unsafe extern "C-unwind" fn build_callback(
    _index: pg_sys::Relation,
    ctid: pg_sys::ItemPointer,
    values: *mut pg_sys::Datum,
    is_null: *mut bool,
    _tuple_is_alive: bool,
    state: *mut std::os::raw::c_void,
) {
    let state = unsafe { &mut *(state as *mut BuildState) };

    if unsafe { *is_null.add(0) } {
        return;
    }

    let doc_id = ctid_to_doc_id(unsafe { *ctid });
    let text: Option<String> =
        unsafe { pgrx::datum::FromDatum::from_datum(*values.add(0), false) };

    if let Some(content) = text {
        let doc = serde_json::json!({
            "id": doc_id,
            "content": content,
        });

        if let Err(e) = state.client.sync_document(&state.collection, &doc_id, &doc) {
            pgrx::warning!("pgaf: failed to sync document {}: {}", doc_id, e);
        } else {
            state.count += 1.0;
        }
    }
}

#[pgrx::pg_guard]
pub unsafe extern "C-unwind" fn ambuildempty(_index_relation: pg_sys::Relation) {
    // No-op for remote index. Unlogged tables will have an empty index
    // after crash recovery, which is acceptable.
}

#[pgrx::pg_guard]
#[allow(clippy::too_many_arguments)]
pub unsafe extern "C-unwind" fn aminsert(
    index_relation: pg_sys::Relation,
    values: *mut pg_sys::Datum,
    is_null: *mut bool,
    heap_tid: pg_sys::ItemPointer,
    _heap_relation: pg_sys::Relation,
    _check_unique: pg_sys::IndexUniqueCheck::Type,
    _index_unchanged: bool,
    _index_info: *mut pg_sys::IndexInfo,
) -> bool {
    if unsafe { *is_null.add(0) } {
        return false;
    }

    let (url, collection) = unsafe { options::get_options(index_relation) };
    let doc_id = ctid_to_doc_id(unsafe { *heap_tid });

    let text: Option<String> =
        unsafe { pgrx::datum::FromDatum::from_datum(*values.add(0), false) };

    if let Some(content) = text {
        let client = AntflyClient::new(&url).unwrap_or_else(|e| {
            pgrx::error!("pgaf: failed to create client: {}", e);
        });

        let doc = serde_json::json!({
            "id": doc_id,
            "content": content,
        });

        if let Err(e) = client.sync_document(&collection, &doc_id, &doc) {
            pgrx::warning!("pgaf: failed to sync document to antfly: {}", e);
        }
    }

    false
}
