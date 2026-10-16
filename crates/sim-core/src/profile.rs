//! Per-phase timing for `World::tick_once`, behind the `profile` feature.
//!
//! `cargo run --release --features profile --example tickbench` prints where a tick's time
//! goes, phase by phase. Without the feature `phase` is a plain call the optimiser removes, so
//! normal builds pay nothing.
//!
//! The numbers are wall-clock on the calling thread. Each phase costs one `Instant::now` pair
//! (tens of nanoseconds), so a phase that runs every tick has a small, even overhead; the
//! shares between phases are what matters.

#[cfg(feature = "profile")]
mod enabled {
    use std::cell::RefCell;
    use std::time::{Duration, Instant};

    thread_local! {
        /// (phase, total time, calls) in the order each phase was first seen.
        static PHASES: RefCell<Vec<(&'static str, Duration, u64)>> = const { RefCell::new(Vec::new()) };
    }

    /// Times `f` as the phase `name`, adding its wall time to that phase's total.
    pub fn phase<R>(name: &'static str, f: impl FnOnce() -> R) -> R {
        let start = Instant::now();
        let out = f();
        let dt = start.elapsed();
        PHASES.with(|p| {
            let mut p = p.borrow_mut();
            match p.iter_mut().find(|(n, ..)| *n == name) {
                Some((_, total, calls)) => {
                    *total += dt;
                    *calls += 1;
                }
                None => p.push((name, dt, 1)),
            }
        });
        out
    }

    /// Forgets every phase so far (start a fresh measurement).
    pub fn reset() {
        PHASES.with(|p| p.borrow_mut().clear());
    }

    /// One line per phase, heaviest first: total ms, share, ns a tick and calls a tick. Empty
    /// before anything is measured.
    pub fn report() -> String {
        PHASES.with(|p| {
            let p = p.borrow();
            let total: Duration = p.iter().map(|(_, d, _)| *d).sum();
            let ticks = p.iter().map(|(_, _, c)| *c).max().unwrap_or(0);
            let ms_total = total.as_secs_f64() * 1000.0;
            let mut rows: Vec<_> = p
                .iter()
                .map(|&(name, d, calls)| {
                    let ms = d.as_secs_f64() * 1000.0;
                    let share = if ms_total == 0.0 { 0.0 } else { 100.0 * ms / ms_total };
                    let ns = d.as_nanos() as f64 / ticks.max(1) as f64;
                    (name, ms, share, ns, calls as f64 / ticks.max(1) as f64)
                })
                .collect();
            rows.sort_by(|a, b| b.1.total_cmp(&a.1));
            let mut out = format!(
                "{:>9} {:>6} {:>9} {:>6}  {}\n",
                "ms", "%", "ns/tick", "calls", "phase"
            );
            for (name, ms, share, ns, calls) in rows {
                out += &format!("{ms:9.1} {share:6.1} {ns:9.1} {calls:6.2}  {name}\n");
            }
            out
        })
    }
}

#[cfg(not(feature = "profile"))]
mod disabled {
    /// No-op: the `profile` feature is off, so the optimiser removes the call.
    #[inline(always)]
    pub fn phase<R>(_name: &'static str, f: impl FnOnce() -> R) -> R {
        f()
    }

    /// No-op: the `profile` feature is off.
    #[inline(always)]
    pub fn reset() {}

    /// Empty: the `profile` feature is off.
    pub fn report() -> String {
        String::new()
    }
}

#[cfg(feature = "profile")]
pub use enabled::{phase, report, reset};
#[cfg(not(feature = "profile"))]
pub use disabled::{phase, report, reset};
