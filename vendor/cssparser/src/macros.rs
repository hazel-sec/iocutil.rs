/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// See docs of the `procedural-masquerade` crate.
define_invoke_proc_macro!(cssparser_internal__invoke_proc_macro);

/// Expands to a `match` expression with string patterns,
/// matching case-insensitively in the ASCII range.
///
/// The patterns must not contain ASCII upper case letters. (They must be already be lower-cased.)
///
/// # Example
///
/// ```rust
/// #[macro_use] extern crate cssparser;
///
/// # fn main() {}  // Make doctest not wrap everythig in its own main
/// # fn dummy(function_name: &String) { let _ =
/// match_ignore_ascii_case! { &function_name,
///     "rgb" => parse_rgb(..),
///     "rgba" => parse_rgba(..),
///     "hsl" => parse_hsl(..),
///     "hsla" => parse_hsla(..),
///     _ => Err(format!("unknown function: {}", function_name))
/// }
/// # ;}
/// # use std::ops::RangeFull;
/// # fn parse_rgb(_: RangeFull) -> Result<(), String> { Ok(()) }
/// # fn parse_rgba(_: RangeFull) -> Result<(), String> { Ok(()) }
/// # fn parse_hsl(_: RangeFull) -> Result<(), String> { Ok(()) }
/// # fn parse_hsla(_: RangeFull) -> Result<(), String> { Ok(()) }
/// ```
#[macro_export]
macro_rules! match_ignore_ascii_case {
    ( $input:expr, $( $match_body:tt )* ) => {
        {
            cssparser_internal__invoke_proc_macro! {
                cssparser_internal__assert_ascii_lowercase__max_len!( $( $match_body )* )
            }

            {
                // MAX_LENGTH is generated by cssparser_internal__assert_ascii_lowercase__max_len
                cssparser_internal__to_lowercase!($input, MAX_LENGTH => lowercase);
                // "A" is a short string that we know is different for every string pattern,
                // since we’ve verified that none of them include ASCII upper case letters.
                match lowercase.unwrap_or("A") {
                    $( $match_body )*
                }
            }
        }
    };
}

/// Define a function `$name(&str) -> Option<&'static $ValueType>`
///
/// The function finds a match for the input string
/// in a [`phf` map](https://github.com/sfackler/rust-phf)
/// and returns a reference to the corresponding value.
/// Matching is case-insensitive in the ASCII range.
///
/// ## Example:
///
/// ```rust
/// #[macro_use] extern crate cssparser;
///
/// # fn main() {}  // Make doctest not wrap everything in its own main
///
/// fn color_rgb(input: &str) -> Option<(u8, u8, u8)> {
///     ascii_case_insensitive_phf_map! {
///         keyword -> (u8, u8, u8) = {
///             "red" => (255, 0, 0),
///             "green" => (0, 255, 0),
///             "blue" => (0, 0, 255),
///         }
///     }
///     keyword(input).cloned()
/// }
#[macro_export]
macro_rules! ascii_case_insensitive_phf_map {
    ($name: ident -> $ValueType: ty = { $( $key: expr => $value: expr ),* }) => {
        ascii_case_insensitive_phf_map!($name -> $ValueType = { $( $key => $value, )* })
    };
    ($name: ident -> $ValueType: ty = { $( $key: expr => $value: expr, )* }) => {
        pub fn $name(input: &str) -> Option<&'static $ValueType> {
            cssparser_internal__invoke_proc_macro! {
                cssparser_internal__phf_map!( ($ValueType) $( $key ($value) )+ )
            }

            {
                cssparser_internal__invoke_proc_macro! {
                    cssparser_internal__max_len!( $( $key )+ )
                }
                // MAX_LENGTH is generated by cssparser_internal__max_len
                cssparser_internal__to_lowercase!(input, MAX_LENGTH => lowercase);
                lowercase.and_then(|s| MAP.get(s))
            }
        }
    }
}

/// Implementation detail of match_ignore_ascii_case! and ascii_case_insensitive_phf_map! macros.
///
/// **This macro is not part of the public API. It can change or be removed between any versions.**
///
/// Define a local variable named `$output`
/// and assign it the result of calling `_internal__to_lowercase`
/// with a stack-allocated buffer of length `$BUFFER_SIZE`.
#[macro_export]
#[doc(hidden)]
macro_rules! cssparser_internal__to_lowercase {
    ($input: expr, $BUFFER_SIZE: expr => $output: ident) => {
        let mut buffer;
        // Safety: `buffer` is only used in `_internal__to_lowercase`,
        // which initializes with `copy_from_slice` the part of the buffer it uses,
        // before it uses it.
        #[allow(unsafe_code)]
        let buffer = unsafe { cssparser_internal__uninit!(buffer, $BUFFER_SIZE) };
        let input: &str = $input;
        let $output = $crate::_internal__to_lowercase(buffer, input);
    };
}

#[cfg(has_std__mem__MaybeUninit)]
#[macro_export]
#[doc(hidden)]
macro_rules! cssparser_internal__uninit {
    ($buffer: ident, $BUFFER_SIZE: expr) => {
        {
            $buffer = ::std::mem::MaybeUninit::<[u8; $BUFFER_SIZE]>::uninit();
            &mut *($buffer.as_mut_ptr())
        }
    }
}

// FIXME: remove this when we require Rust 1.36
#[cfg(not(has_std__mem__MaybeUninit))]
#[macro_export]
#[doc(hidden)]
macro_rules! cssparser_internal__uninit {
    ($buffer: ident, $BUFFER_SIZE: expr) => {
        {
            $buffer = ::std::mem::uninitialized::<[u8; $BUFFER_SIZE]>();
            &mut $buffer
        }
    }
}

/// Implementation detail of match_ignore_ascii_case! and ascii_case_insensitive_phf_map! macros.
///
/// **This function is not part of the public API. It can change or be removed between any verisons.**
///
/// If `input` is larger than buffer, return `None`.
/// Otherwise, return `input` ASCII-lowercased, using `buffer` as temporary space if necessary.
#[doc(hidden)]
#[allow(non_snake_case)]
pub fn _internal__to_lowercase<'a>(buffer: &'a mut [u8], input: &'a str) -> Option<&'a str> {
    if let Some(buffer) = buffer.get_mut(..input.len()) {
        if let Some(first_uppercase) = input.bytes().position(|byte| matches!(byte, b'A'..=b'Z')) {
            buffer.copy_from_slice(input.as_bytes());
            buffer[first_uppercase..].make_ascii_lowercase();
            // `buffer` was initialized to a copy of `input` (which is &str so well-formed UTF-8)
            // then lowercased (which preserves UTF-8 well-formedness)
            unsafe { Some(::std::str::from_utf8_unchecked(buffer)) }
        } else {
            // Input is already lower-case
            Some(input)
        }
    } else {
        // Input is longer than buffer, which has the length of the longest expected string:
        // none of the expected strings would match.
        None
    }
}

#[cfg(feature = "dummy_match_byte")]
macro_rules! match_byte {
    ($value:expr, $($rest:tt)* ) => {
        match $value {
            $(
                $rest
            )+
        }
    };
}
