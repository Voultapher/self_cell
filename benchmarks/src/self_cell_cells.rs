use self_cell::self_cell;

pub type I32Ref<'a> = &'a i32;

self_cell!(
    pub struct I32Cell {
        owner: i32,

        #[covariant]
        dependent: I32Ref,
    }
);

pub type Ast<'a> = Vec<&'a str>;

self_cell!(
    pub struct StringCell {
        owner: String,

        #[covariant]
        dependent: Ast,
    }
);
