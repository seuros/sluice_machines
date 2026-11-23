//! Align one checkpoint across fixed inputs. No storage, timers or I/O.
#![no_std]

use core::num::NonZeroU64;
use periscope_machines::Issuer;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refusal {
    Busy,
    Stale,
    UnknownInput,
    NotAligned,
    Exhausted,
}

/// The new input to pause, if any, and whether all inputs are now aligned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Alignment {
    pub pause: Option<u8>,
    pub aligned: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Checkpoint {
    id: u64,
    held: u64,
}

/// A single barrier coordinator. Every set bit names one participating input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sluice {
    inputs: NonZeroU64,
    issuer: Issuer,
    pending: Option<Checkpoint>,
}

impl Sluice {
    /// Use a fresh scope on restart if old receipts can still arrive.
    #[must_use]
    pub const fn new(scope: u16, inputs: NonZeroU64) -> Self {
        Self {
            inputs,
            issuer: Issuer::new(scope),
            pending: None,
        }
    }

    /// Start an alignment under a fresh checkpoint ID.
    ///
    /// # Errors
    /// Refuses another in-flight checkpoint or exhausted IDs.
    pub const fn begin(&mut self) -> Result<u64, Refusal> {
        if self.pending.is_some() {
            return Err(Refusal::Busy);
        }
        let Ok(id) = self.issuer.issue() else {
            return Err(Refusal::Exhausted);
        };
        self.pending = Some(Checkpoint { id, held: 0 });
        Ok(id)
    }

    /// Record a barrier. The adapter must stop this input at its barrier
    /// before accepting `aligned` as permission to begin capture.
    ///
    /// # Errors
    /// Refuses a stale checkpoint or an input outside the participant set.
    pub fn arrive(&mut self, id: u64, input: u8) -> Result<Alignment, Refusal> {
        let checkpoint = self
            .pending
            .as_mut()
            .filter(|c| c.id == id)
            .ok_or(Refusal::Stale)?;
        let bit = 1_u64
            .checked_shl(u32::from(input))
            .filter(|bit| self.inputs.get() & bit != 0)
            .ok_or(Refusal::UnknownInput)?;
        let pause = (checkpoint.held & bit == 0).then_some(input);
        checkpoint.held |= bit;
        Ok(Alignment {
            pause,
            aligned: checkpoint.held == self.inputs.get(),
        })
    }

    /// Acknowledge durable capture and return all inputs to resume.
    ///
    /// # Errors
    /// Refuses a stale or incompletely aligned checkpoint.
    pub fn captured(&mut self, id: u64) -> Result<u64, Refusal> {
        self.release(id, true)
    }

    /// Abandon the checkpoint and return all inputs to resume.
    ///
    /// # Errors
    /// Refuses a stale checkpoint.
    pub fn abort(&mut self, id: u64) -> Result<u64, Refusal> {
        self.release(id, false)
    }

    fn release(&mut self, id: u64, require_aligned: bool) -> Result<u64, Refusal> {
        let checkpoint = self.pending.filter(|c| c.id == id).ok_or(Refusal::Stale)?;
        if require_aligned && checkpoint.held != self.inputs.get() {
            return Err(Refusal::NotAligned);
        }
        self.pending = None;
        Ok(checkpoint.held)
    }
}

#[cfg(test)]
mod tests;
