macro_rules! assert_ok {
    ($x:expr) => {
        {
            match $x {
                Ok(result) => result,
                Err(error) => panic!("Expected Ok value, got Err({:?}", error),
            }
        }
    }
}
