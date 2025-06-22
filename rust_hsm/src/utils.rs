use crate::state::StateId;

pub(crate) fn get_state_choice<States: From<u16>>(state_id: &StateId) -> States {
    States::from(*state_id.get_id())
}

pub(crate) fn resolve_state_name<States: std::fmt::Display + From<u16>>(
    state_id: &StateId,
) -> String {
    format!("{}", get_state_choice::<States>(state_id))
}

/// Get the full path to a function from crate downwards
macro_rules! get_function_path {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        name.strip_suffix("::f").unwrap()
    }};
}
pub(crate) use get_function_path;

// Returns the function name along with its direct owner!
macro_rules! get_function_name {
    () => {{
        let path: &'static str = crate::utils::get_function_path!();
        let split_res = path.split_terminator("::").collect::<Vec<&str>>();
        match split_res.len().checked_sub(2) {
            None => path.to_string(),
            Some(second_last_index) => split_res.split_at(second_last_index).1.join("::")
        }

    }.to_string()};
}
pub(crate) use get_function_name;

#[cfg(test)]
mod tests {
    use super::*;

    // Note: I use strum to simplify this / so I don't need to hand roll the conversion for From / converting to string.
    // In your use cases, you can implement From anyway you wish.
    // Note 2: I use PartialEq to simplify comparison in the test, but it is not strictly necessary.
    #[repr(u16)]
    #[derive(strum::FromRepr, PartialEq, strum::Display)]
    enum FakeStateType {
        StateA = 1,
        StateB = 2,
        StateC = 3,
        Invalid = 4,
    }

    impl From<u16> for FakeStateType {
        fn from(state_id: u16) -> Self {
            match Self::from_repr(state_id) {
                Some(val) => val,
                None => Self::Invalid,
            }
        }
    }

    #[test]
    fn test_function_macros() {
        let path = get_function_path!();
        let name = get_function_name!();
        assert_eq!(path, "rust_hsm::utils::tests::test_function_macros");
        assert_eq!(name, "tests::test_function_macros");
    }

    #[test]
    fn test_get_state_choice() {
        assert!(get_state_choice::<FakeStateType>(&StateId::new(1)) == FakeStateType::StateA);
        assert!(get_state_choice::<FakeStateType>(&StateId::new(2)) == FakeStateType::StateB);
        assert!(get_state_choice::<FakeStateType>(&StateId::new(3)) == FakeStateType::StateC);
        assert!(get_state_choice::<FakeStateType>(&StateId::new(4)) == FakeStateType::Invalid);
    }

    #[test]
    fn test_resolve_state_name() {
        assert!(resolve_state_name::<FakeStateType>(&StateId::new(1)) == "StateA");
        assert!(resolve_state_name::<FakeStateType>(&StateId::new(2)) == "StateB");
        assert!(resolve_state_name::<FakeStateType>(&StateId::new(3)) == "StateC");
        assert!(resolve_state_name::<FakeStateType>(&StateId::new(4)) == "Invalid");
    }
}
