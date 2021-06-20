use std::str::FromStr;

use self_cell::self_cell;

type I32Ref<'a> = &'a i32;

self_cell!(
    pub struct I32Cell {
        owner: i32,

        #[covariant]
        dependent: I32Ref,
    }
);

pub fn i32_cell_new(x: i32) -> I32Cell {
    I32Cell::new(x, |o| o)
}

pub fn i32_cell_try_new_ok(x: i32) -> Result<I32Cell, Box<u32>> {
    I32Cell::try_new(x, |o| Ok(o))
}

pub fn i32_list(n: i32) -> i32 {
    let mut side_effect = 0;

    let cells = (0..n).into_iter().map(|x| I32Cell::new(x, |x| x));

    for cell in cells {
        side_effect += if **cell.borrow_dependent() % 66 == 0 {
            *cell.borrow_owner()
        } else {
            **cell.borrow_dependent() + 8
        };
    }

    side_effect
}

// The list functions mostly test L1 access
// Let's also test pseudo random access.

pub fn i32_random(n: i32) -> i32 {
    let mut side_effect = 0;

    let cells = (0..n)
        .into_iter()
        .map(|x| I32Cell::new(x, |x| x))
        .collect::<Vec<_>>();

    while side_effect < n * 2 {
        side_effect += **cells[(side_effect as usize) % (cells.len() - 66)].borrow_dependent() + 1;
    }

    side_effect
}

type Ast<'a> = Vec<&'a str>;

pub fn ast_from_string(body: &String) -> Ast {
    body.split("+").filter(|x| x.len() > 1).collect()
}

self_cell!(
    pub struct StringCell {
        owner: String,

        #[covariant]
        dependent: Ast,
    }
);

pub fn string_cell_new(x: String) -> StringCell {
    StringCell::new(x, ast_from_string)
}

pub fn string_cell_try_new_ok(x: String) -> Result<StringCell, Box<u32>> {
    StringCell::try_new(x, |o| Ok(ast_from_string(o)))
}

pub fn string_list(n: i32) -> i32 {
    let mut side_effect = 0;

    let cells = (0..n)
        .into_iter()
        .map(|x| StringCell::new(x.to_string(), ast_from_string));

    for cell in cells {
        let val = cell
            .borrow_dependent()
            .iter()
            .map(|x| i32::from_str(x).unwrap())
            .sum();

        side_effect += if val % 66 == 0 {
            cell.borrow_owner().len() as i32
        } else {
            val
        };
    }

    side_effect
}

// string 1m is too long

pub fn string_random(n: i32) -> i32 {
    let mut side_effect = 0;

    let cells = (0..n)
        .into_iter()
        .map(|x| StringCell::new(x.to_string(), ast_from_string))
        .collect::<Vec<_>>();

    while side_effect < n * 2 {
        side_effect += cells[(side_effect as usize) % (cells.len() - 66)]
            .borrow_dependent()
            .iter()
            .map(|x| i32::from_str(x).unwrap())
            .sum::<i32>()
            + 1;
    }

    side_effect
}
