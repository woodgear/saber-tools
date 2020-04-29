mod failure_backtrace;
pub trait FailureErrorExt {
    fn pretty_log(&self) -> String;
}

impl FailureErrorExt for failure::Error {
    fn pretty_log(&self) -> String {
        let backtrace_msg = failure_backtrace::pretty_backtrace(self.backtrace());
        return  format!("{:?}\n{}",self.to_string(),backtrace_msg);
    }
}
