#![allow(dead_code)]

extern crate alloc;

#[path = "../server/src/record.rs"]
mod record;
#[path = "../server/src/ledger.rs"]
mod ledger;

#[cfg(test)]
#[path = "ledger_case.rs"]
mod ledger_case;
#[cfg(test)]
#[path = "codec_case.rs"]
mod codec_case;
#[cfg(test)]
#[path = "declaration_case.rs"]
mod declaration_case;
