use itertools::Itertools;
use nuts::{nut00::Proof, traits::Unit};

use crate::types::{
    NodeUrl,
    compact_wad::{CompactKeysetProofs, CompactProof, CompactWad},
};

pub fn create_from_proofs<U: Unit>(
    node_url: NodeUrl,
    unit: U,
    memo: Option<String>,
    proofs: Vec<Proof>,
) -> CompactWad<U> {
    let compact_proofs = proofs
        .into_iter()
        .chunk_by(|p| p.keyset_id)
        .into_iter()
        .map(|(keyset_id, proofs)| CompactKeysetProofs {
            keyset_id,
            proofs: proofs
                .map(|p| CompactProof {
                    amount: p.amount,
                    secret: p.secret,
                    c: p.c,
                })
                .collect(),
        })
        .collect();

    CompactWad {
        node_url,
        unit,
        memo,
        proofs: compact_proofs,
    }
}
