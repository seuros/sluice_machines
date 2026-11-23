use super::*;

#[test]
fn missing_input_aborts_and_late_capture_cannot_release_the_next_barrier() -> Result<(), Refusal> {
    let mut sluice = Sluice::new(1, NonZeroU64::MIN.saturating_add(2));
    let first = sluice.begin()?;
    assert_eq!(
        sluice.arrive(first, 1),
        Ok(Alignment {
            pause: Some(1),
            aligned: false
        })
    );
    assert_eq!(
        sluice.arrive(first, 1),
        Ok(Alignment {
            pause: None,
            aligned: false
        })
    );
    assert_eq!(sluice.captured(first), Err(Refusal::NotAligned));
    assert_eq!(sluice.abort(first), Ok(2));
    let second = sluice.begin()?;
    assert_eq!(sluice.captured(first), Err(Refusal::Stale));
    assert!(!sluice.arrive(second, 0)?.aligned);
    assert!(sluice.arrive(second, 1)?.aligned);
    assert_eq!(sluice.captured(second), Ok(3));
    Ok(())
}
