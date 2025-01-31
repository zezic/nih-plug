// nih-plug: plugins, but rewritten in Rust
// Copyright (C) 2022 Robbert van der Helm
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

/// Write something to the STDERR stream.
///
/// XXX: I don't think we need all of the log crate just for some simple logging, but maybe consider
///      integrating some other crate with this function if we need to log to some other place than
///      STDERR or if it needs to be done in release builds and we should thus try to avoid
///      allocations.
#[macro_export]
macro_rules! nih_log {
    ($format:expr $(, $($args:tt)*)?) => (
        eprintln!(concat!("[", file!(), ":", line!(), "] ", $format), $($($args)*)?)
    );
}

/// A `debug_assert!()` analogue that prints the error with line number information instead of
/// panicking.
///
/// TODO: Detect if we're running under a debugger, and trigger a break if we are
#[macro_export]
macro_rules! nih_debug_assert {
    ($cond:expr $(,)?) => (
        if cfg!(debug_assertions) && !$cond {
            nih_log!(concat!("Debug assertion failed: ", stringify!($cond)));
        }
    );
    ($cond:expr, $format:expr $(, $($args:tt)*)?) => (
        if cfg!(debug_assertions) && !$cond {
            nih_log!(concat!("Debug assertion failed: ", stringify!($cond), ", ", $format), $($($args)*)?);
        }
    );
}

/// An unconditional debug assertion failure, for if the condition has already been checked
/// elsewhere.
#[macro_export]
macro_rules! nih_debug_assert_failure {
    () => (
        if cfg!(debug_assertions) {
            nih_log!("Debug assertion failed");
        }
    );
    ($format:expr $(, $($args:tt)*)?) => (
        if cfg!(debug_assertions) {
            nih_log!(concat!("Debug assertion failed: ", $format), $($($args)*)?);
        }
    );
}

/// A `debug_assert_eq!()` analogue that prints the error with line number information instead of
/// panicking.
#[macro_export]
macro_rules! nih_debug_assert_eq {
    ($left:expr, $right:expr $(,)?) => (
        if cfg!(debug_assertions) && $left != $right {
            nih_log!(concat!("Debug assertion failed: ", stringify!($left), " != ", stringify!($right)));
        }
    );
    (left:expr, $right:expr, $format:expr $(, $($args:tt)*)?) => (
        if cfg!(debug_assertions) && $left != $right  {
            nih_log!(concat!("Debug assertion failed: ", stringify!($left), " != ", stringify!($right), ", ", $format), $($($args)*)?);
        }
    );
}

/// A `debug_assert_ne!()` analogue that prints the error with line number information instead of
/// panicking.
#[macro_export]
macro_rules! nih_debug_assert_ne {
    ($left:expr, $right:expr $(,)?) => (
        if cfg!(debug_assertions) && $left == $right {
            nih_log!(concat!("Debug assertion failed: ", stringify!($left), " == ", stringify!($right)));
        }
    );
    (left:expr, $right:expr, $format:expr $(, $($args:tt)*)?) => (
        if cfg!(debug_assertions) && $left == $right  {
            nih_log!(concat!("Debug assertion failed: ", stringify!($left), " == ", stringify!($right), ", ", $format), $($($args)*)?);
        }
    );
}
