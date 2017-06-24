macro_rules! impl_convert_for_error {
    ($from:path, $to:path) => (
        impl convert::From<$from> for Error {
            fn from(err: $from) -> Error {
                $to(err)
            }
        }
    );
}

