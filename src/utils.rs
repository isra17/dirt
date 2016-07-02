use std::io;
use std::io::Write;

pub trait LogError<E> {
    fn log_err<'a, F>(self, f: F) -> Self where F: FnOnce(&E) -> String;
}

impl<T, E> LogError<E> for Result<T, E> {
    fn log_err<'a, F>(self, f: F) -> Self
        where F: FnOnce(&E) -> String
    {
        match self {
            Ok(_) => (),
            Err(ref e) => {
                io::stderr().write(f(e).as_bytes()).unwrap();
            }
        };
        return self;
    }
}
