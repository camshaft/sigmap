use signal_hook::iterator::exfiltrator::SignalOnly;
use signal_hook::{flag, iterator::SignalsInfo};
use std::{
    collections::HashMap,
    process::Command,
    sync::{atomic::AtomicBool, Arc},
    thread::spawn,
};

type Result<T, E = std::io::Error> = core::result::Result<T, E>;

macro_rules! input_err {
    ($($arg:tt)*) => {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, format!($($arg)*))
    };
}

fn main() -> Result<()> {
    let args = std::env::args();
    // skip the binary name
    let mut args = args.skip(1).peekable();

    let mut froms = vec![];
    let mut map = HashMap::new();

    while let Some(entry) = args.peek().and_then(|arg| parse(arg)) {
        let (from, to) = entry?;
        froms.push(from);
        map.insert(from, to);
        let _ = args.next();
    }

    // Make sure double CTRL+C and similar kills
    let term_now = Arc::new(AtomicBool::new(false));

    for sig in signal_hook::consts::TERM_SIGNALS {
        flag::register_conditional_shutdown(*sig, 1, term_now.clone())?;
        flag::register(*sig, term_now.clone())?;
    }

    let mut signals = SignalsInfo::<SignalOnly>::new(&froms)?;

    let command = args
        .next()
        .ok_or_else(|| input_err!("missing subcommand"))?;
    let mut command = Command::new(command);
    command.args(args);
    let mut child = command.spawn()?;

    let pid = child.id();
    spawn(move || {
        for signal in &mut signals {
            // map from one signal to another
            let mapped = map.get(&signal).copied().unwrap_or(signal);
            eprintln!("sending {} (mapped from {})", mapped, signal);
            let _ = send_signal(pid as _, mapped);
        }
    });

    let status = child.wait()?;

    if let Some(code) = status.code() {
        std::process::exit(code);
    } else {
        std::process::exit(1);
    }
}

fn send_signal(pid: i32, sig: i32) -> Result<()> {
    extern "C" {
        fn kill(pid: i32, sig: i32) -> i32;
    }

    if unsafe { kill(pid, sig) } >= 0 {
        Ok(())
    } else {
        Err(std::io::Error::last_os_error())
    }
}

fn parse(n: &str) -> Option<Result<(i32, i32)>> {
    n.split_once(':').map(|(from, to)| {
        let from = parse_signal(from)?;
        let to = parse_signal(to)?;
        Ok((from, to))
    })
}

fn parse_signal(sig: &str) -> Result<i32> {
    use signal_hook::consts::signal::*;

    macro_rules! signals {
        ($(($signal:ident, $short:ident)),* $(,)?) => {{
            match sig {
                $(
                    stringify!($signal) | stringify!($short) => return Ok($signal),
                )*
                _ => {}
            }

            match sig.parse() {
                $(
                    Ok($signal) => return Ok($signal),
                )*
                _ => {}
            }

            Err(input_err!("unknown signal: {}", sig))
        }}
    }

    signals!(
        (SIGABRT, ABRT),
        (SIGALRM, ALRM),
        (SIGBUS, BUS),
        (SIGCHLD, CHLD),
        (SIGCONT, CONT),
        (SIGFPE, FPE),
        (SIGHUP, HUP),
        (SIGILL, ILL),
        (SIGINT, INT),
        (SIGIO, IO),
        (SIGKILL, KILL),
        (SIGPIPE, PIPE),
        (SIGPROF, PROF),
        (SIGQUIT, QUIT),
        (SIGSEGV, SEGV),
        (SIGSTOP, STOP),
        (SIGSYS, SYS),
        (SIGTERM, TERM),
        (SIGTRAP, TRAP),
        (SIGTSTP, TSTP),
        (SIGTTIN, TTIN),
        (SIGTTOU, TTOU),
        (SIGURG, URG),
        (SIGUSR1, USR),
        (SIGUSR2, USR),
        (SIGVTALRM, VTALRM),
        (SIGWINCH, WINCH),
        (SIGXCPU, XCPU),
        (SIGXFSZ, XFSZ),
    )
}
