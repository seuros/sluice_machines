# Sluice Machines

> Close each gate at its own marker. Don't call it a cut until every gate is shut.

Sluice aligns one checkpoint across a fixed, nonempty set of inputs. The inputs
reach their barriers at different times. Hold each one at its marker so records
from after that marker can't leak into the checkpoint.

## The Flight Plan

1. **Name the inputs.** Build `Sluice` with a nonzero bitmask of participant IDs
   (0–63). Call `begin()` to get a scoped Periscope checkpoint ID, and attach
   that ID to every barrier. A second `begin()` while one is active is `Busy`.
2. **Hold the input at its barrier.** Call `arrive(id, input)` and act on
   `Alignment::pause` before processing any later record from that input.
   A repeat arrival for that input doesn't request a second pause.
3. **Capture once, with every hold in place.** `aligned` means all barriers
   have arrived; the adapter must apply the final pause before it captures.
   Capture once per ID, even if a repeat arrival also reports `aligned`.
   Call `captured(id)` only after the cut is durable, then resume the returned
   input mask.
4. **If an input never arrives, abort explicitly.** `abort(id)` returns only
   the inputs held so far; resume those before starting the next checkpoint.
   A late barrier or capture receipt for the abandoned ID is `Stale`, not a
   vote for the next checkpoint or permission to release its inputs.

## What's Yours

Sluice has no clock, transport or storage. The adapter owns barrier ordering,
actual holds and releases, capture durability, timeouts, and recovery of paused
inputs after a crash. Use a fresh scope on restart if old receipts can arrive.

**Status:** Unpublished `no_std` core. Foundation composes it with Offset for
the cut and Snapshot for the manifest in a local barrier simulation.
