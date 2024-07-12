use std::fmt::Display;

use crate::state::StateId;

pub(crate) fn get_state_choice<StateType: Display + Into<u16> + From<u16> + Clone>(
    state_id: &StateId,
) -> StateType {
    StateType::from(*state_id.get_id())
}

pub(crate) fn resolve_state_name<StateType: Display + Into<u16> + From<u16> + Clone>(
    state_id: &StateId,
) -> String {
    get_state_choice::<StateType>(state_id)
        .to_string()
        .clone()
        .to_string()
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
        // let mut name: String = "";
        let path: &'static str = crate::utils::get_function_path!();
        let split_res = path.split_terminator("::").collect::<Vec<&str>>();
        // println!("{:?}", split_res);
        // &split_res.join("")
        match split_res.len().checked_sub(2) {
            None => path.to_string(),
            Some(second_last_index) => split_res.split_at(second_last_index).1.join("::")
            // Some(second_last_index) => {
                // split_res.split_at(second_last_index).1.join("")
                // (split_res[second_last_index].to_string() + split_res[second_last_index - 1]).as_str()
            // }
        }
        // name

        // split_res.last().unwrap()
    }.to_string()};
}
pub(crate) use get_function_name;

#[cfg(test)]
mod tests {
    #[test]
    fn foo() {
        let path = get_function_path!();
        let name = get_function_name!();
        assert_eq!(path, "rust_hsm::utils::tests::foo");
        assert_eq!(name, "tests::foo");
    }
}
