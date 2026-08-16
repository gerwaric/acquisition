// REVIEW MUTATION (SD-R8 re-close, F24 range): an ordinary-spelling,
// token-bearing construction forgery in an in-crate descendant of `mock`,
// written in a .rs file outside the pin's src/ scan set.
use crate::mock::MockEvidence;

#[test]
fn review_mutation_f25_probe_out_of_scan_ordinary_spelling_is_silent() {
    let forged = MockEvidence {
        observations: Vec::new(),
        state_changes: Vec::new(),
    };
    assert!(forged.observations().is_empty());
    assert!(forged.state_changes().is_empty());
}
