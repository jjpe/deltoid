//!


#[macro_export]
macro_rules! ensure_eq {
    ($left:expr, $right:expr) => {{
        let (lstr, rstr) = (stringify!($left), stringify!($right));
        ensure!(
            $left == $right,
            "violation: {} == {}\n{} == {}\n{} == {}",
            lstr, rstr,
            lstr, $left,
            rstr, $right
        )
    }};
}

#[macro_export]
macro_rules! ensure_ne {
    ($left:expr, $right:expr) => {{
        let (lstr, rstr) = (stringify!($left), stringify!($right));
        ensure!(
            $left != $right,
            "violation: {} != {}\n{} == {}\n{} == {}",
            lstr, rstr,
            lstr, $left,
            rstr, $right
        )
    }};
}

#[macro_export]
macro_rules! ensure_gt {
    ($left:expr, $right:expr) => {{
        let (lstr, rstr) = (stringify!($left), stringify!($right));
        ensure!(
            $left > $right,
            "violation: {} > {}\n{} == {}\n{} == {}",
            lstr, rstr,
            lstr, $left,
            rstr, $right
        )
    }};
}

#[macro_export]
macro_rules! ensure_lt {
    ($left:expr, $right:expr) => {{
        let (lstr, rstr) = (stringify!($left), stringify!($right));
        ensure!(
            $left < $right,
            "violation: {} < {}\n{} == {}\n{} == {}",
            lstr, rstr,
            lstr, $left,
            rstr, $right
        )
    }};
}

#[macro_export]
macro_rules! ensure_ge {
    ($left:expr, $right:expr) => {{
        let (lstr, rstr) = (stringify!($left), stringify!($right));
        ensure!(
            $left >= $right,
            "violation: {} >= {}\n{} == {}\n{} == {}",
            lstr, rstr,
            lstr, $left,
            rstr, $right
        )
    }};
}

#[macro_export]
macro_rules! ensure_le {
    ($left:expr, $right:expr) => {{
        let (lstr, rstr) = (stringify!($left), stringify!($right));
        ensure!(
            $left <= $right,
            "violation: {} <= {}\n{} == {}\n{} == {}",
            lstr, rstr,
            lstr, $left,
            rstr, $right
        )
    }};
}

#[macro_export]
macro_rules! ensure {
    ($predicate:expr $(, $fmt:expr $(, $args:expr)*)? ) => {
        if $predicate {
            $crate::error::DeltaResult::Ok(())
        } else {
            Err($crate::error::DeltaError::FailedToEnsure {
                predicate: stringify!($predicate),
                msg: {
                    #[allow(unused)] let mut msg = String::new();
                    $(  msg = format!($fmt $(, $args)*);  )?
                    msg
                },
                file: file!(),
                line: line!(),
                column: column!(),
            })
        }
    };
}

#[macro_export]
macro_rules! bug_detected {
    ($($fmt:expr $(, $args:expr)*)?) => {
        Err($crate::error::DeltaError::BugDetected {

            msg: { #[allow(redundant_semicolon)] {
                #[allow(unused)] let mut msg = String::new();
                $(  msg = format!($fmt $(, $args)*);  )? ;
                msg
            }},
            file: file!(),
            line: line!(),
            column: column!(),
        })
    };
}



pub type DeltaResult<T> = Result<T, DeltaError>;

#[derive(Clone, Debug)]
pub enum DeltaError {
    ExpectedValue,
    FailedToEnsure {
        predicate: &'static str,
        msg: String,
        file: &'static str,
        line: u32,
        column: u32
    },
    BugDetected {
        msg: String,
        file: &'static str,
        line: u32,
        column: u32
    },
}
