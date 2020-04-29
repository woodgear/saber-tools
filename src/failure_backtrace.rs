
// copy from https://github.com/athre0z/color-backtrace

mod color_backtrace {
    
}
struct FakeBacktrace {
    internal: FakeInternalBacktrace,
}

struct FakeInternalBacktrace {
    backtrace: Option<MaybeResolved>,
}

struct MaybeResolved {
    _resolved: std::sync::Mutex<bool>,
    backtrace: std::cell::UnsafeCell<backtrace::Backtrace>,
}

pub unsafe fn backdoortrace(_opaque: &failure::Backtrace) -> Option<&backtrace::Backtrace> {
    let _ = format!("{}", _opaque); // forces resolution
    let no_longer_opaque: &FakeBacktrace =
        { &*(_opaque as *const failure::Backtrace as *const FakeBacktrace) }; // unsafe
    if let Some(bt) = &no_longer_opaque.internal.backtrace {
        let bt = { &*bt.backtrace.get() }; // unsafe
        return Some(bt);
    }

    None
}
use std::path::PathBuf;
#[derive(Debug)]
struct Frame {
    name: Option<String>,
    lineno: Option<u32>,
    filename: Option<PathBuf>,
}

impl Frame {
    fn show(&self) -> String {
        let name = self.name.clone().unwrap_or_default();
        let mut names = name.split("::").collect::<Vec<_>>();
        names.truncate(names.len() - 1);
        let func_name = names.join("::");
        let lineno = self.lineno.unwrap_or_default();
        let filename = self.filename.clone().unwrap_or_default();
        let filename = filename.to_string_lossy().to_string();
        format!("{}:{} {}", filename,lineno,func_name)
    }
}

impl Frame {
    fn is_dependency_code(&self) -> bool {
        const SYM_PREFIXES: &[&str] = &[
            "std::",
            "core::",
            "backtrace::backtrace::",
            "_rust_begin_unwind",
            "color_traceback::",
            "__rust_",
            "___rust_",
            "__pthread",
            "_main",
            "main",
            "__scrt_common_main_seh",
            "BaseThreadInitThunk",
            "_start",
            "__libc_start_main",
            "start_thread",
        ];

        // Inspect name.
        if let Some(ref name) = self.name {
            if SYM_PREFIXES.iter().any(|x| name.starts_with(x)) {
                return true;
            }
        }

        const FILE_PREFIXES: &[&str] = &[
            "/rustc/",
            "src/libstd/",
            "src/libpanic_unwind/",
            "src/libtest/",
        ];

        // Inspect filename.
        if let Some(ref filename) = self.filename {
            let filename = filename.to_string_lossy();
            if FILE_PREFIXES.iter().any(|x| filename.starts_with(x))
                || filename.contains("/.cargo/registry/src/")
            {
                return true;
            }
        }

        false
    }

    /// Heuristically determine whether a frame is likely to be a post panic
    /// frame.
    ///
    /// Post panic frames are frames of a functions called after the actual panic
    /// is already in progress and don't contain any useful information for a
    /// reader of the backtrace.
    fn is_post_panic_code(&self) -> bool {
        const SYM_PREFIXES: &[&str] = &[
            "_rust_begin_unwind",
            "core::result::unwrap_failed",
            "core::panicking::panic_fmt",
            "color_backtrace::create_panic_handler",
            "std::panicking::begin_panic",
            "begin_panic_fmt",
            "failure::backtrace::Backtrace::new",
            "backtrace::capture",
            "failure::error_message::err_msg",
            "<failure::error::Error as core::convert::From<F>>::from",
        ];

        match self.name.as_ref() {
            Some(name) => SYM_PREFIXES.iter().any(|x| name.starts_with(x)),
            None => false,
        }
    }
}

fn pretty_std_backtrace(backtrace: &backtrace::Backtrace) -> String {
    let frames: Vec<_> = backtrace
        .frames()
        .iter()
        .flat_map(|frame| frame.symbols())
        .map(|sym| Frame {
            name: sym.name().map(|x| x.to_string()),
            lineno: sym.lineno(),
            filename: sym.filename().map(|x| x.into()),
        })
        .collect();
    let mut msg = "".to_string();
    for f in frames.iter() {
        if f.is_dependency_code() || f.is_post_panic_code() {
            continue;
        }
        msg += &format!("{}\n",f.show());
    }

    return  msg;
}

pub fn pretty_backtrace(backtrace: &failure::Backtrace) -> String {
    let std_backtrace = unsafe { backdoortrace(backtrace) };
    match std_backtrace {
        Some(b) => {
            return pretty_std_backtrace(b);
        }
        None => {
            return "".to_string();
        }
    }
}
