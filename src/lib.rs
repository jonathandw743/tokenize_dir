pub use tokenize_dir_macros::tokenize_dir;

mod to_constraints;

pub use to_constraints::ToConstraints;

/// returns None for unconstrained
/// 
/// returns Some(Vec<usize>) where the all the values satisfy all the cosntraints
/// where a cosntraint is skipped if it cannot be satisfied given the previous constraints
pub fn solve_constraints_nonstrict<'a>(
    mut constraints: impl Iterator<Item = impl AsRef<[usize]>>,
) -> Option<Vec<usize>> {
    let Some(first_constraint) = constraints.next() else {
        return None;
    };
    let first_nonempty_constraint = if first_constraint.as_ref().is_empty() {
        if let Some(possible_files) = constraints
            .find(|token_associated_files| !token_associated_files.as_ref().is_empty())
        {
            possible_files
        } else {
            return Some(Vec::new());
        }
    } else {
        first_constraint
    };
    let mut partial_solution = Vec::from(first_nonempty_constraint.as_ref());
    while let Some(constraint) = constraints.next() {
        let constraint = constraint.as_ref();
        if partial_solution.len() == 1 {
            break;
        }
        if constraint.is_empty() {
            continue;
        }
        if partial_solution[partial_solution.len() - 1] < constraint[0] {
            continue;
        }
        if partial_solution[0] > constraint[constraint.len() - 1] {
            continue;
        }
        let mut i = 0;
        let mut j = 0;
        if partial_solution.first().unwrap() > constraint.first().unwrap() {
            let x = partial_solution[0];
            let mut b = constraint.len() / 2;
            while b > 0 {
                while j + b < constraint.len() && constraint[j + b] <= x {
                    j += b;
                }
                b /= 2;
            }
        } else
        // if possible_files.first().unwrap() < token_associated_files.first().unwrap()
        {
            let x = constraint[0];
            let mut b = partial_solution.len() / 2;
            while b > 0 {
                while i + b < partial_solution.len() && partial_solution[i + b] <= x {
                    i += b;
                }
                b /= 2;
            }
        }
        let mut next_partial_solution = Vec::with_capacity(partial_solution.len());
        // let mut next_partial_solution = Vec::new();
        while i < partial_solution.len() && j < constraint.len() {
            if partial_solution[i] == constraint[j] {
                next_partial_solution.push(partial_solution[i]);
                i += 1;
                j += 1;
            } else if partial_solution[i] < constraint[j] {
                i += 1;
            } else {
                j += 1;
            }
        }
        if !next_partial_solution.is_empty() {
            partial_solution = next_partial_solution;
        }
    }
    Some(partial_solution)
}

/// returns None for no value
/// 
/// returns Some(usize) where there is a value
pub fn first_value_nonstrict<'a>(
    mut constraints: impl Iterator<Item = impl AsRef<[usize]>>,
) -> Option<usize> {
    let Some(first_constraint) = constraints.next() else {
        return Some(0);
    };
    let first_nonempty_constraint = if first_constraint.as_ref().is_empty() {
        if let Some(possible_files) = constraints
            .find(|token_associated_files| !token_associated_files.as_ref().is_empty())
        {
            possible_files
        } else {
            return None;
        }
    } else {
        first_constraint
    };
    let mut partial_solution = Vec::from(first_nonempty_constraint.as_ref());
    while let Some(constraint) = constraints.next() {
        let constraint = constraint.as_ref();
        if partial_solution.len() == 1 {
            break;
        }
        if constraint.is_empty() {
            continue;
        }
        if partial_solution[partial_solution.len() - 1] < constraint[0] {
            continue;
        }
        if partial_solution[0] > constraint[constraint.len() - 1] {
            continue;
        }
        let mut i = 0;
        let mut j = 0;
        if partial_solution.first().unwrap() > constraint.first().unwrap() {
            let x = partial_solution[0];
            let mut b = constraint.len() / 2;
            while b > 0 {
                while j + b < constraint.len() && constraint[j + b] <= x {
                    j += b;
                }
                b /= 2;
            }
        } else
        // if possible_files.first().unwrap() < token_associated_files.first().unwrap()
        {
            let x = constraint[0];
            let mut b = partial_solution.len() / 2;
            while b > 0 {
                while i + b < partial_solution.len() && partial_solution[i + b] <= x {
                    i += b;
                }
                b /= 2;
            }
        }
        let mut next_partial_solution = Vec::with_capacity(partial_solution.len());
        // let mut next_partial_solution = Vec::new();
        while i < partial_solution.len() && j < constraint.len() {
            if partial_solution[i] == constraint[j] {
                next_partial_solution.push(partial_solution[i]);
                i += 1;
                j += 1;
            } else if partial_solution[i] < constraint[j] {
                i += 1;
            } else {
                j += 1;
            }
        }
        if !next_partial_solution.is_empty() {
            partial_solution = next_partial_solution;
        }
    }
    Some(partial_solution[0])
}


/// returns None for unconstrained
/// 
/// returns Some(Vec<usize>) where the all the values satisfy all the cosntraints
pub fn solve_constraints_strict<'a>(
    mut constraints: impl Iterator<Item = impl AsRef<[usize]>>,
) -> Option<Vec<usize>> {
    let Some(first_constraint) = constraints.next() else {
        return None;
    };
    let mut partial_solution = Vec::from(first_constraint.as_ref());
    while let Some(constraint) = constraints.next() {
        if partial_solution.is_empty() {
            break;
        }
        let constraint = constraint.as_ref();
        let mut i = 0;
        let mut j = 0;
        if partial_solution.first().unwrap() > constraint.first().unwrap() {
            let x = partial_solution[0];
            let mut b = constraint.len() / 2;
            while b > 0 {
                while j + b < constraint.len() && constraint[j + b] <= x {
                    j += b;
                }
                b /= 2;
            }
        } else
        // if possible_files.first().unwrap() < token_associated_files.first().unwrap()
        {
            let x = constraint[0];
            let mut b = partial_solution.len() / 2;
            while b > 0 {
                while i + b < partial_solution.len() && partial_solution[i + b] <= x {
                    i += b;
                }
                b /= 2;
            }
        }
        let mut next_partial_solution = Vec::with_capacity(partial_solution.len());
        // let mut next_partial_solution = Vec::new();
        while i < partial_solution.len() && j < constraint.len() {
            if partial_solution[i] == constraint[j] {
                next_partial_solution.push(partial_solution[i]);
                i += 1;
                j += 1;
            } else if partial_solution[i] < constraint[j] {
                i += 1;
            } else {
                j += 1;
            }
        }
        partial_solution = next_partial_solution;
    }
    Some(partial_solution)
}

/// returns None no value satisfies
/// 
/// returns Some(usize) when there is a value
pub fn first_value_strict<'a>(
    mut constraints: impl Iterator<Item = impl AsRef<[usize]>>,
) -> Option<usize> {
    let Some(first_constraint) = constraints.next() else {
        return Some(0);
    };
    let mut partial_solution = Vec::from(first_constraint.as_ref());
    while let Some(constraint) = constraints.next() {
        if partial_solution.is_empty() {
            break;
        }
        let constraint = constraint.as_ref();
        let mut i = 0;
        let mut j = 0;
        if partial_solution.first().unwrap() > constraint.first().unwrap() {
            let x = partial_solution[0];
            let mut b = constraint.len() / 2;
            while b > 0 {
                while j + b < constraint.len() && constraint[j + b] <= x {
                    j += b;
                }
                b /= 2;
            }
        } else
        // if possible_files.first().unwrap() < token_associated_files.first().unwrap()
        {
            let x = constraint[0];
            let mut b = partial_solution.len() / 2;
            while b > 0 {
                while i + b < partial_solution.len() && partial_solution[i + b] <= x {
                    i += b;
                }
                b /= 2;
            }
        }
        let mut next_partial_solution = Vec::with_capacity(partial_solution.len());
        // let mut next_partial_solution = Vec::new();
        while i < partial_solution.len() && j < constraint.len() {
            if partial_solution[i] == constraint[j] {
                next_partial_solution.push(partial_solution[i]);
                i += 1;
                j += 1;
            } else if partial_solution[i] < constraint[j] {
                i += 1;
            } else {
                j += 1;
            }
        }
        partial_solution = next_partial_solution;
    }
    partial_solution.first().map(|x| *x)
}