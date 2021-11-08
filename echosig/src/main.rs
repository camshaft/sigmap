use signal_hook::{
    consts::TERM_SIGNALS,
    flag,
    iterator::{exfiltrator::SignalOnly, SignalsInfo},
};
use std::io::Result;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

fn main() -> Result<()> {
    let term_now = Arc::new(AtomicBool::new(false));
    for sig in TERM_SIGNALS {
        flag::register_conditional_shutdown(*sig, 1, term_now.clone())?;
        flag::register(*sig, term_now.clone())?;
    }

    let mut signals = SignalsInfo::<SignalOnly>::new(TERM_SIGNALS)?;

    for signal in &mut signals {
        eprintln!("GOT {}", signal);
        std::process::exit(signal);
    }

    Ok(())
}
