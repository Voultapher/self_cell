use ouroboros::self_referencing;

pub type I32Ref<'a> = &'a i32;

#[self_referencing]
struct I32CellImpl {
    owner: Box<i32>,

    #[borrows(owner)]
    #[covariant]
    dependent: I32Ref<'this>,
}

pub struct I32Cell(I32CellImpl);

impl I32Cell {
    pub fn new(owner: i32, dependent_builder: impl for<'a> FnOnce(&'a i32) -> I32Ref<'a>) -> Self {
        Self(
            I32CellImplBuilder {
                owner: Box::new(owner),
                dependent_builder,
            }
            .build(),
        )
    }

    pub fn try_new<E>(
        owner: i32,
        dependent_builder: impl for<'a> FnOnce(&'a i32) -> Result<I32Ref<'a>, E>,
    ) -> Result<Self, E> {
        Ok(Self(
            I32CellImplTryBuilder {
                owner: Box::new(owner),
                dependent_builder,
            }
            .try_build()?,
        ))
    }

    pub fn borrow_owner(&self) -> &i32 {
        self.0.borrow_owner()
    }

    pub fn borrow_dependent(&self) -> &I32Ref {
        self.0.borrow_dependent()
    }
}

pub type Ast<'a> = Vec<&'a str>;

#[self_referencing]
pub struct StringCellImpl {
    owner: String,

    #[borrows(owner)]
    #[covariant]
    dependent: Ast<'this>,
}

pub struct StringCell(StringCellImpl);

impl StringCell {
    pub fn new(owner: String, dependent_builder: impl for<'a> FnOnce(&'a str) -> Ast<'a>) -> Self {
        Self(
            StringCellImplBuilder {
                owner,
                dependent_builder,
            }
            .build(),
        )
    }

    pub fn try_new<E>(
        owner: String,
        dependent_builder: impl for<'a> FnOnce(&'a str) -> Result<Ast<'a>, E>,
    ) -> Result<Self, E> {
        Ok(Self(
            StringCellImplTryBuilder {
                owner,
                dependent_builder,
            }
            .try_build()?,
        ))
    }

    pub fn borrow_owner(&self) -> &String {
        self.0.borrow_owner()
    }

    pub fn borrow_dependent(&self) -> &Ast {
        self.0.borrow_dependent()
    }
}
