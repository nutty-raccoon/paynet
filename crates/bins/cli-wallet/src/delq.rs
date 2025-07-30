use anyhow::Result;
use nuts::traits::Unit;
use wallet::types::compact_wad::{CompactWad, DleqVerificationResult};

/// Verifies DLEQ proofs in a wad and prints verification summary.
pub async fn verify_and_print_dleq_proofs<U: Unit>(wad: &CompactWad<U>) -> Result<()> {
    println!("\nVerifying DLEQ proofs...");

    let mut node_client = wallet::connect_to_node(&wad.node_url).await?;

    match wallet::verify_compact_wad_dleq_proofs(wad, &mut node_client).await {
        Ok(verification_result) => {
            match &verification_result {
                DleqVerificationResult::AllValid => {
                    println!("DLEQ Verification: ✓ PASSED (All proofs are valid)")
                }
                DleqVerificationResult::AllNone => {
                    println!("DLEQ Verification: No DLEQ verification")
                }
                DleqVerificationResult::SomeNotInvalid(errors) => {
                    println!(
                        "DLEQ Verification: ✗ FAILED ({}/{} proofs valid)",
                        errors.len(),
                        wad.proofs.len()
                    )
                }
            }

            println!("\nVerification Details:");
            if let DleqVerificationResult::SomeNotInvalid(errors) = &verification_result {
                for status in errors {
                    if let Some(error) = &status.error {
                        let proof = &wad.proofs()[status.index];
                        println!(
                            "Proof {} ({} {}): ✗ DLEQ Invalid ({})",
                            status.index + 1,
                            proof.amount,
                            wad.unit(),
                            error
                        );
                    }
                }
            }
        }
        Err(e) => return Err(e.into()),
    }

    Ok(())
}
