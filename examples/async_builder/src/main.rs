use self_cell::self_cell;

type Dependent<'a> = &'a str;

self_cell!(
    struct AsyncStringCell {
        owner: String,

        #[covariant, async_builder]
        dependent: Dependent,
    }
);

async fn as_part_of_async_program(val: String) -> AsyncStringCell {
    let start_idx = 6; // This value will be captured into the `async` closure.
    AsyncStringCell::new(val, async |owner| &owner[start_idx..]).await
}

fn non_async_fn(cell: &AsyncStringCell) -> usize {
    // Only the cell construction becomes `async` when using `async_builder` option.
    // The other functions remain sync.
    cell.borrow_dependent().len()
}

fn main() {
    smol::block_on(async {
        let cell = as_part_of_async_program("Tua - Vorstadt".to_string()).await;
        assert_eq!(non_async_fn(&cell), 8);
        println!(
            "Owner: {}, dependent: {}",
            cell.borrow_owner(),
            cell.borrow_dependent()
        );
    });
}
