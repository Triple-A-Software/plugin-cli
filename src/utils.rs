use std::process;

pub trait SoftPanic<T> {
    fn soft_expect(self, msg: &str) -> T;
}

impl<T, E> SoftPanic<T> for Result<T, E> {
    fn soft_expect(self, msg: &str) -> T {
        match self {
            Ok(t) => t,
            Err(_) => {
                println!("{}", msg);
                process::exit(1);
            }
        }
    }
}

impl<T> SoftPanic<T> for Option<T> {
    fn soft_expect(self, msg: &str) -> T {
        match self {
            Some(t) => t,
            None => {
                println!("{}", msg);
                process::exit(1);
            }
        }
    }
}
