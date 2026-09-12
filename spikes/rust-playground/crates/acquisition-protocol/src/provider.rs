//! Which provider a process is for — part of the handshake (C10): a
//! daemon's `hello` names the provider it serves, and a client uses only
//! a daemon on the provider it wants itself. The names and the one knob
//! that selects between them live here; the providers themselves (URLs,
//! client id, user-agent, keyring service) are the daemon's
//! (`acquisition-daemon/src/provider.rs`), because only the daemon sends.
//!
//! C88 — the real provider is the default and the mock is opted into:
//! `ACQ_PROVIDER=mock` selects the in-process mock; unset or `ggg` is the
//! real API. The mock default was the ladder's first rung — traffic off
//! GGG by structure while the limiter was unproven — and outlived the
//! ruling that retired its premise (C14): what protects the relationship
//! is the one gate every send passes through, not callers kept away.
//! Meanwhile every offline read of the owner's own facts sat behind a
//! flag. The old knob, `ACQ_GGG`, is refused rather than ignored
//! ([`check_environment`]): a stale export or an old script fails loudly
//! at start, never silently in the wrong mode.

/// The in-process mock provider: the test double and the rehearsal
/// stage, selected by `ACQ_PROVIDER=mock`.
pub const MOCK: &str = "mock";

/// The real GGG API, under the existing "acquisition" registration: the
/// default.
pub const GGG: &str = "ggg";

/// The knob: `mock`, `ggg`, or unset (`ggg`). Any other value is refused
/// at start.
pub const KNOB: &str = "ACQ_PROVIDER";

/// The retired knob (C88): its presence in the environment, whatever the
/// value, is refused at start.
const RETIRED_KNOB: &str = "ACQ_GGG";

/// True when this process (daemon or CLI) is in real-GGG mode: the
/// default, unless `ACQ_PROVIDER=mock`.
pub fn ggg_mode() -> bool {
    !std::env::var(KNOB).is_ok_and(|v| v == MOCK)
}

/// The provider this process wants a daemon to serve: [`MOCK`] under
/// `ACQ_PROVIDER=mock`, [`GGG`] otherwise. What the client compares the
/// daemon's `hello` against.
pub fn wanted() -> &'static str {
    if ggg_mode() { GGG } else { MOCK }
}

/// The refusal every binary makes at start (C88), before it reads the
/// knob: `ACQ_GGG` present at all — the knob is gone and its old meaning
/// is now the default, so a leftover export must not run in the wrong
/// mode silently — or `ACQ_PROVIDER` set to a word that names no
/// provider. `Ok` means [`wanted`] reads exactly what was asked.
pub fn check_environment() -> Result<(), String> {
    if std::env::var_os(RETIRED_KNOB).is_some() {
        return Err(format!(
            "{RETIRED_KNOB} is no longer read: the real provider is the default and \
             {KNOB}=mock selects the mock (C88); unset {RETIRED_KNOB}"
        ));
    }
    match std::env::var(KNOB) {
        Err(std::env::VarError::NotPresent) => Ok(()),
        Ok(v) if v == MOCK || v == GGG => Ok(()),
        Ok(v) => Err(format!(
            "{KNOB}={v:?} names no provider: {MOCK} or {GGG} (unset is {GGG}, C88)"
        )),
        Err(e) => Err(format!("{KNOB} is unreadable: {e}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV: Mutex<()> = Mutex::new(());

    /// C88: unset is the real provider; `mock` opts into the mock; the
    /// retired knob and an unknown word are refused, each naming the
    /// remedy.
    #[test]
    fn c88_the_default_is_ggg_the_mock_is_opted_into_and_the_old_knob_is_refused() {
        let _env = ENV.lock().unwrap();
        // SAFETY: `ENV` is held; this is the module's only environment test.
        unsafe {
            std::env::remove_var(RETIRED_KNOB);
            std::env::remove_var(KNOB);
        }
        assert_eq!(wanted(), GGG);
        assert!(ggg_mode());
        assert_eq!(check_environment(), Ok(()));
        unsafe { std::env::set_var(KNOB, "mock") };
        assert_eq!(wanted(), MOCK);
        assert!(!ggg_mode());
        assert_eq!(check_environment(), Ok(()));
        unsafe { std::env::set_var(KNOB, "ggg") };
        assert_eq!(wanted(), GGG);
        assert_eq!(check_environment(), Ok(()));
        unsafe { std::env::set_var(KNOB, "Mock") };
        let err = check_environment().unwrap_err();
        assert!(
            err.contains("names no provider") && err.contains("C88"),
            "{err}"
        );
        unsafe {
            std::env::remove_var(KNOB);
            std::env::set_var(RETIRED_KNOB, "0");
        }
        let err = check_environment().unwrap_err();
        assert!(
            err.contains("ACQ_GGG is no longer read") && err.contains("ACQ_PROVIDER=mock"),
            "{err}"
        );
        unsafe { std::env::remove_var(RETIRED_KNOB) };
    }
}
